use rustc_ast::{AttrKind, Attribute, PathSegment};
use rustc_ast::ast::{Item, ItemKind};
use rustc_ast::token::CommentKind;
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};
use rustc_session::{declare_tool_lint, impl_lint_pass};
use rustc_span::symbol::Symbol;
use rustc_span::{BytePos, FileName, Span};

use std::collections::HashMap;
use std::ops::Range;
use std::path::PathBuf;

declare_tool_lint! {
    pub doc_stripper::DOC_STRIPPER,
    Warn,
    "invalid `doc_stripper` attribute"
}
impl_lint_pass!(DocStripper => [DOC_STRIPPER]);

#[derive(Debug)]
pub struct ItemDocInfo {
    // For example: `a::b::c`.
    path: Vec<String>,
    doc_comments: Vec<(Symbol, Span)>,
}

fn item_to_str(item: &Item) -> &'static str {
    match item.kind {
        ItemKind::Const(..) => "const",
        ItemKind::GlobalAsm(..) | ItemKind::Delegation(..) => {
            panic!("{:?} is not supported by `doc_stripper`", item.kind)
        }
        ItemKind::Enum(..) => "enum",
        ItemKind::ExternCrate(..) => "crate",
        ItemKind::Fn(..) => "fn",
        ItemKind::ForeignMod(..) => "foreign_mod",
        ItemKind::Impl(..) => "impl",
        ItemKind::MacCall(..) => "macro call",
        ItemKind::MacroDef(..) => "macro",
        ItemKind::Mod(..) => "mod",
        ItemKind::Static(..) => "static",
        ItemKind::Struct(..) => "struct",
        ItemKind::Trait(..) => "trait",
        ItemKind::TraitAlias(..) => "trait_alias",
        ItemKind::TyAlias(..) => "ty_alias",
        ItemKind::Union(..) => "union",
        ItemKind::Use(..) => "use",
    }
}

impl ItemDocInfo {
    fn new(path: Vec<String>, doc_comments: Vec<(Symbol, Span)>) -> Self {
        Self {
            path,
            doc_comments,
        }
    }
}

pub struct DocStripper {
    items_with_docs: Vec<ItemDocInfo>,
    current_path: Vec<String>,
    output_file: PathBuf,
}

pub fn run_doc_stripper(output_file: PathBuf) -> Box<dyn EarlyLintPass> {
    Box::new(DocStripper {
        items_with_docs: Vec::new(),
        current_path: Vec::new(),
        output_file,
    })
}

fn attr_path(attr: &Attribute) -> Option<&[PathSegment]> {
    match &attr.kind {
        AttrKind::Normal(normal) => Some(normal.item.path.segments.as_slice()),
        AttrKind::DocComment(..) => None,
    }
}

impl EarlyLintPass for DocStripper {
    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &rustc_ast::Item) {
        let mut doc_attrs = Vec::new();
        let mut ignoring = false;
        for attr in &item.attrs {
            if let Some(doc) = attr.doc_str() {
                if !ignoring {
                    if matches!(attr.kind, AttrKind::DocComment(CommentKind::Line, _)) {
                        doc_attrs.push((doc, attr.span.with_hi(attr.span.hi() + BytePos(1))));
                    } else {
                        doc_attrs.push((doc, attr.span));
                    }
                }
            } else if let Some([name1, name2]) = attr_path(attr)
                && name1.ident.name.as_str() == "doc_stripper"
            {
                match name2.ident.name.as_str() {
                    "ignore_next" => ignoring = true,
                    "strip_next" => ignoring = false,
                    _ => cx.span_lint(
                        DOC_STRIPPER,
                        attr.span,
                        "invalid `doc_stripper` attribute",
                        |diag| {
                            diag.help("existing attributes: `ignore_next` and `strip_next`");
                        },
                    ),
                }
            }
        }
        self.current_path
            .push(format!("{} {}", item_to_str(item), item.ident.name));
        if !doc_attrs.is_empty() {
            self.items_with_docs
                .push(ItemDocInfo::new(self.current_path.clone(), doc_attrs));
        }
    }

    fn check_item_post(&mut self, cx: &EarlyContext<'_>, _: &rustc_ast::Item) {
        self.current_path.pop();
    }

    fn check_crate_post(&mut self, cx: &EarlyContext<'_>, _: &rustc_ast::Crate) {
        let source_map = cx.sess().source_map();
        let mut files = HashMap::new();
        let mut out_md_docs = String::new();

        for item in &self.items_with_docs {
            if !out_md_docs.is_empty() {
                out_md_docs.push('\n');
            }
            out_md_docs.push_str(&format!("<!-- {} -->\n\n", item.path.join("/")));
            for (doc, span) in &item.doc_comments {
                let filename = source_map.span_to_filename(*span);
                if let FileName::Real(ref filename) = filename
                    && let Some(local_path) = filename.local_path()
                {
                    files.entry(local_path.to_path_buf()).or_insert(Vec::new()).push(*span);
                }
                out_md_docs.push_str(doc.as_str());
                out_md_docs.push('\n');
            }
        }
        let output_file = match self.output_file.canonicalize() {
            Ok(o) => o,
            Err(_) => self.output_file.clone(),
        };
        eprintln!("Storing stripped documentation into `{}`", output_file.display());
        std::fs::write(output_file, out_md_docs).unwrap();
        for (file_path, mut spans) in files {
            spans.sort_unstable_by(|sp1, sp2| sp1.lo().partial_cmp(&sp2.lo()).unwrap());
            spans.reverse();
            let start = source_map.lookup_char_pos(spans[0].lo()).file.start_pos.0 as usize;
            let mut out = std::fs::read(&file_path).unwrap();
            for span in spans {
                // Remove a range.
                out.splice(
                    Range {
                        start: span.lo().0 as usize - start,
                        end: span.hi().0 as usize - start
                    },
                    [],
                );
            }
            println!("Updating `{}`", file_path.display());
            std::fs::write(file_path, out).unwrap();
        }
    }
}
