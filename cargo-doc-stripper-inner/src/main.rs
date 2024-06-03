#![feature(rustc_private)] // This feature must be added so we can use compiler APIs.
#![feature(let_chains)]

// We need to import them like this otherwise it doesn't work.
extern crate rustc_ast;
extern crate rustc_lint;
extern crate rustc_session;
extern crate rustc_span;

use rustc_ast::{Mutability, TyKind};
use rustc_ast::ast::{Item, ItemKind};

use rustc_lint::LintStore;
use rustc_tools::with_lints;

use std::path::PathBuf;

mod doc_regen;
mod doc_stripper;

const COMMENT_START: &str = "<!-- <stripper> ";
const DEFAULT_FILE_NAME: &str = "documentation.md";

fn ty_to_str(ty_kind: &TyKind) -> String {
    match ty_kind {
        TyKind::Slice(ty) => format!("&[{}]", ty_to_str(&ty.kind)),
        TyKind::Array(ty, _) => format!("[{}]", ty_to_str(&ty.kind)),
        TyKind::Ptr(ty) => format!(
            "*{} {}",
            if matches!(ty.mutbl, Mutability::Not) {
                "const"
            } else {
                "mut"
            },
            ty_to_str(&ty.ty.kind),
        ),
        TyKind::Ref(_, ty) => format!(
            "&{}{}",
            if matches!(ty.mutbl, Mutability::Not) {
                ""
            } else {
                "mut "
            },
            ty_to_str(&ty.ty.kind),
        ),
        // FIXME: render it better?
        TyKind::BareFn(..) => "<bare_fn>".into(),
        TyKind::Never => "!".into(),
        TyKind::Tup(tys) => format!(
            "({})",
            tys.iter()
                .map(|ty| ty_to_str(&ty.kind))
                .collect::<Vec<_>>()
                .join(", "),
        ),
        TyKind::AnonStruct(..) => "<anon struct>".into(),
        TyKind::AnonUnion(..) => "<anon union>".into(),
        TyKind::Path(qself, path) => {
            let qself = if let Some(qself) = qself {
                format!("<{}> as ", ty_to_str(&qself.ty.kind))
            } else {
                String::new()
            };
            format!(
                "{qself}{}",
                path.segments
                    .iter()
                    .map(|seg| seg.ident.name.as_str())
                    .collect::<Vec<_>>()
                    .join("::"),
            )
        }
        TyKind::TraitObject(..) => "<trait_object>".into(),
        TyKind::ImplTrait(..) => "<impl_trait>".into(),
        // FIXME: improve rendering?
        TyKind::Pat(..) => "<pat>".into(),
        TyKind::Paren(..)
        | TyKind::Typeof(..)
        | TyKind::Infer
        | TyKind::ImplicitSelf
        | TyKind::MacCall(..)
        | TyKind::CVarArgs
        | TyKind::Dummy
        | TyKind::Err(..) => String::new(),
    }
}

fn item_to_str(item: &Item) -> String {
    let kind = match item.kind {
        ItemKind::Const(..) => "const",
        ItemKind::GlobalAsm(..) | ItemKind::Delegation(..) => {
            panic!("{:?} is not supported by `doc_stripper`", item.kind)
        }
        ItemKind::Enum(..) => "enum",
        ItemKind::ExternCrate(..) => "crate",
        ItemKind::Fn(..) => "fn",
        ItemKind::ForeignMod(..) => "foreign_mod",
        ItemKind::Impl(ref impl_) => {
            let trait_ = if let Some(ref trait_) = impl_.of_trait {
                format!(
                    "trait: {} ",
                    trait_.path.segments.last().unwrap().ident.name.as_str()
                )
            } else {
                String::new()
            };
            return format!("impl <{trait_}type: {}>", ty_to_str(&impl_.self_ty.kind));
        }
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
    };
    format!("{kind} {}", item.ident.name)
}

#[allow(non_upper_case_globals)]
fn main() {
    let mut cargo_args = std::env::args().skip(2).collect::<Vec<_>>();

    static mut output_file: Option<PathBuf> = None;
    static mut input_file: Option<PathBuf> = None;
    let mut is_strip: bool = false;
    let mut is_regen: bool = false;
    let mut strip_by_default = false;

    let mut to_remove = Vec::new();
    let mut iter_args = cargo_args.iter().enumerate();
    while let Some((pos, arg)) = iter_args.next() {
        match arg.as_str() {
            "--output-file" => {
                match iter_args.next() {
                    Some((_, out)) => unsafe { output_file = Some(out.into()) },
                    None => {
                        eprintln!("Expecting file path after `--output-file`");
                        std::process::exit(1);
                    }
                }
                to_remove.push(pos);
                to_remove.push(pos + 1);
            }
            "--input-file" => {
                match iter_args.next() {
                    Some((_, out)) => unsafe { input_file = Some(out.into()) },
                    None => {
                        eprintln!("Expecting file path after `--input-file`");
                        std::process::exit(1);
                    }
                }
                to_remove.push(pos);
                to_remove.push(pos + 1);
            }
            "--strip-by-default" => {
                strip_by_default = true;
                to_remove.push(pos);
            }
            "--strip" => {
                is_strip = true;
                to_remove.push(pos);
            }
            "--regen" => {
                is_regen = true;
                to_remove.push(pos);
            }
            "--help" => {
                println!(
                    "Help for `cargo doc-stripper`

 --strip             : Strip documentation from provided crate
 --regen             : Regenerate documentation from provided crate
 --output-file [PATH]: Path where the strip documentation will be stored
 --input-file [PATH] : Path from where the stripped documentation will be restored
 --strip-by-default  : Unless this option is used, only documentation with
                       `#[doc_stripper::strip_next]` will be stripped.
 --help              : Show this help"
                );
                std::process::exit(0);
            }
            _ => {}
        }
    }

    if is_strip && is_regen {
        eprintln!("Both `--strip` and `--regen` options cannot be used at the same time");
        std::process::exit(1);
    } else if !is_strip && !is_regen {
        eprintln!("At least one of `--strip` or `--regen` options should be used");
        std::process::exit(1);
    }

    while let Some(index) = to_remove.pop() {
        cargo_args.remove(index);
    }

    rustc_tools::cargo_integration(&cargo_args, move |args| {
        with_lints(args, vec![], move |store: &mut LintStore| {
            if is_strip {
                store.register_early_pass(move || {
                    unsafe {
                        doc_stripper::run_doc_stripper(
                            output_file
                                .clone()
                                .unwrap_or_else(|| DEFAULT_FILE_NAME.into()),
                            strip_by_default,
                        )
                    }
                });
            } else {
                store.register_early_pass(move || {
                    doc_regen::run_doc_regen(unsafe {
                        input_file
                            .clone()
                            .unwrap_or_else(|| DEFAULT_FILE_NAME.into())
                    })
                });
            }
        })
        .expect("with_lints failed");
    })
    .expect("cargo_integration failed");
}
