use rustc_ast::ast::Item;
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};
use rustc_session::{declare_tool_lint, impl_lint_pass};
use rustc_span::{FileName, Span};

use std::collections::HashMap;
use std::fmt::Write;
use std::ops::Range;
use std::path::PathBuf;

pub fn run_doc_regen(input_file: PathBuf) -> Box<dyn EarlyLintPass> {
    println!(
        "Regenerating stripped documentation from `{}`",
        input_file.display()
    );

    Box::new(DocRegenerator::new(input_file))
}

declare_tool_lint! {
    pub doc_stripper::DOC_REGENERATOR,
    Warn,
    "cannot regenerate documentation"
}
impl_lint_pass!(DocRegenerator => [DOC_REGENERATOR]);

fn prepare_doc(mut doc: &str) -> String {
    doc = doc.trim_end();
    while let Some(new_doc) = doc.strip_prefix('\n') {
        doc = new_doc;
    }
    doc.split('\n')
        .fold(String::with_capacity(doc.len()), |mut acc, line| {
            write!(acc, "///{line}\n").unwrap();
            acc
        })
}

#[derive(Debug)]
struct InsertInfo {
    pos: Span,
    insert_ignore_next: bool,
    doc: String,
}

#[derive(Debug)]
pub struct DocRegenerator {
    // Key is the path of the item and the value is the documentation.
    items_with_docs: HashMap<String, String>,
    insert_at: Vec<InsertInfo>,
    current_path: Vec<String>,
}

impl DocRegenerator {
    fn new(input_file: PathBuf) -> Self {
        let content = match std::fs::read_to_string(&input_file) {
            Ok(content) => content,
            Err(err) => {
                eprintln!("Failed to read `{}`: {err:?}", input_file.display());
                std::process::exit(1);
            }
        };
        let mut items_with_docs = HashMap::new();

        for part in content.split(crate::COMMENT_START) {
            let mut parts = part.splitn(2, " -->");
            if let Some(path) = parts.next()
                && let Some(doc) = parts.next()
            {
                items_with_docs.insert(path.to_string(), prepare_doc(doc));
            }
        }
        Self {
            items_with_docs,
            current_path: Vec::new(),
            insert_at: Vec::new(),
        }
    }
}

impl EarlyLintPass for DocRegenerator {
    fn check_item(&mut self, _: &EarlyContext<'_>, item: &Item) {
        let mut insert_ignore_next = false;
        let insert_pos = match item.attrs.iter().find(|attr| attr.doc_str().is_some()) {
            Some(attr) => {
                insert_ignore_next = true;
                attr.span.lo()
            }
            None => item
                .attrs
                .iter()
                .next()
                .map(|attr| attr.span.lo())
                .unwrap_or(item.span.lo()),
        };
        self.current_path.push(crate::item_to_str(item));
        if let Some(doc) = self.items_with_docs.remove(&self.current_path.join("/")) {
            self.insert_at.push(InsertInfo {
                pos: item.span.with_lo(insert_pos),
                insert_ignore_next,
                doc,
            });
        } else if insert_ignore_next {
            // Even if we have no documented item matching this path, if there is documentation,
            // we definitely want to add `#[doc_stripper::ignore_next]` to prevent it from being
            // removed when stripping afterwards.
            self.insert_at.push(InsertInfo {
                pos: item.span.with_lo(insert_pos),
                insert_ignore_next,
                doc: String::new(),
            });
        }
    }

    fn check_item_post(&mut self, _: &EarlyContext<'_>, _: &rustc_ast::Item) {
        self.current_path.pop();
    }

    fn check_crate_post(&mut self, cx: &EarlyContext<'_>, _: &rustc_ast::Crate) {
        let source_map = cx.sess().source_map();
        let mut files = HashMap::new();

        for insert_info in &self.insert_at {
            let filename = source_map.span_to_filename(insert_info.pos);
            if let FileName::Real(ref filename) = filename
                && let Some(local_path) = filename.local_path()
            {
                files
                    .entry(local_path.to_path_buf())
                    .or_insert(Vec::new())
                    .push(insert_info);
            }
        }
        for (file_path, mut inserters) in files {
            inserters.sort_unstable_by(|i1, i2| i1.pos.lo().partial_cmp(&i2.pos.lo()).unwrap());
            inserters.reverse();

            let start = source_map
                .lookup_char_pos(inserters[0].pos.lo())
                .file
                .start_pos
                .0 as usize;
            let mut out = std::fs::read(&file_path).unwrap();
            for inserter in inserters {
                let pos = inserter.pos.lo().0 as usize - start;
                let range = Range {
                    start: pos,
                    end: pos,
                };
                // Insert string as bytes at the given index.
                if !inserter.doc.is_empty() {
                    out.splice(range.clone(), inserter.doc.clone().into_bytes());
                }
                if inserter.insert_ignore_next {
                    out.splice(range, *b"#[doc_stripper::ignore_next]\n");
                }
            }
            println!("Updating `{}`", file_path.display());
            std::fs::write(file_path, out).unwrap();
        }
    }
}
