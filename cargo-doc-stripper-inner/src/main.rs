#![feature(rustc_private)] // This feature must be added so we can use compiler APIs.
#![feature(let_chains)]

// We need to import them like this otherwise it doesn't work.
extern crate rustc_ast;
extern crate rustc_lint;
extern crate rustc_session;
extern crate rustc_span;

use rustc_lint::LintStore;
use rustc_tools::with_lints;

use std::path::PathBuf;

mod doc_stripper;

fn main() {
    let mut cargo_args = std::env::args().skip(2).collect::<Vec<_>>();

    static mut output_file: Option<PathBuf> = None;
    static mut input_file: Option<PathBuf> = None;
    static mut is_strip: bool = false;
    static mut is_regen: bool = false;

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
            "--strip" => {
                unsafe { is_strip = true ; }
                to_remove.push(pos);
            }
            "--regen" => {
                unsafe { is_regen = true; }
                to_remove.push(pos);
            }
            "--help" => {
                println!("Help for `cargo doc-stripper`");
                print!("\n\n

 --strip             : Strip documentation from provided crate
 --regen             : Regenerate documentation from provided crate
 --output-file [PATH]: Path where the strip documentation will be stored
 --input-file [PATH] : Path from where the stripped documentation will be restored
 --help              : Show this help
");
                std::process::exit(0);
            }
            _ => {}
        }
    }

    if unsafe { is_strip && is_regen } {
        eprintln!("Both `--strip` and `--regen` options cannot be used at the same time");
        std::process::exit(1);
    } else if unsafe { !is_strip && !is_regen } {
        eprintln!("At least one of `--strip` or `--regen` options should be used");
        std::process::exit(1);
    }

    while let Some(index) = to_remove.pop() {
        cargo_args.remove(index);
    }

    const default_file_name: &str = "documentation.md";

    rustc_tools::cargo_integration(&cargo_args, move |args| {
        with_lints(args, vec![], move |store: &mut LintStore| {
            if unsafe { is_strip } {
                store.register_early_pass(move || doc_stripper::run_doc_stripper(unsafe { output_file.clone().unwrap_or_else(|| default_file_name.into()) }));
            } else {
                // to be done.
            }
        })
        .expect("with_lints failed");
    })
    .expect("cargo_integration failed");
}
