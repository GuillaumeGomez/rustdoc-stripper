use std::fs::File;
use std::io::{BufRead, Write};
use std::path::Path;
use std::{env, io};

use clap::{AppSettings, CommandFactory, Parser};

// use stripper_lib::regenerate::regenerate_doc_comments;
use stripper_lib::{strip_comments, OUTPUT_COMMENT_FILE};

struct ExecOptions {
    stdout_output: bool,
    strip: bool,
    regenerate: bool,
    ignore_macros: bool,
    ignore_doc_commented: bool,
}

// #[derive(Parser)]
// #[clap(
//     global_setting(AppSettings::NoAutoVersion),
//     bin_name = "cargo doc-stripper",
//     about = "This utility extract/regenerate doc comments from/into source code."
// )]
// pub struct Opts {
//     /// No output printed to stdout
//     #[clap(short = 'q', long = "quiet")]
//     quiet: bool,

//     /// Use verbose output
//     #[clap(short = 'v', long = "verbose")]
//     verbose: bool,

//     /// Print doc-stripper version and exit
//     #[clap(long = "version")]
//     version: bool,

//     /// Specify package to format
//     #[clap(
//         short = 'p',
//         long = "package",
//         value_name = "package",
//         multiple_values = true
//     )]
//     packages: Vec<String>,

//     /// Specify path to Cargo.toml
//     #[clap(long = "manifest-path", value_name = "manifest-path")]
//     manifest_path: Option<String>,

//     /// Format all packages, and also their local path-based dependencies
//     #[clap(long = "all")]
//     format_all: bool,

//     #[clap(long = "strip", short = 's')]
//     strip: bool,

//     #[clap(long = "regenerate", short = 'r')]
//     regenerate: bool,
// }

fn check_options(args: &mut ExecOptions, to_change: char) -> bool {
    if to_change == 's' {
        args.strip = true;
    } else {
        args.regenerate = true;
    }
    if args.regenerate && args.strip {
        println!("You cannot strip and regenerate at the same time!");
        println!("Rerun with -h option for more information");
        false
    } else {
        true
    }
}

fn print_help() {
    println!(
        r#"Available options for rustdoc-stripper:
    -h | --help                : Displays this help
    -s | --strip               : Strips the specified folder's files and create
                                 a file with doc comments (comments.md by default)
    -g | --regenerate          : Recreate files with doc comments from reading
                                 doc comments file (comments.md by default)
    -n | --no-file-output      : Display doc comments directly on stdout
    -i | --ignore [filename]   : Ignore the specified file, can be repeated as much
                                 as needed, only used when stripping files, ignored
                                 otherwise
    -d | --dir [directory]     : Specify a directory path to work on, optional
    -v | --verbose             : Activate verbose mode
    -f | --force               : Remove confirmation demands
    -m | --ignore-macros       : Macros in hierarchy will be ignored (so only macros
                                 with doc comments will appear in the comments file)
    -o | --comment-file        : Specify the file where you want to save/load doc
                                 comments
    -x | --ignore-doc-commented: When regenerating doc comments, if doc comments
                                 are already present, stored doc comment won't be
                                 regenerated

By default, rustdoc-stripper is run with -s option:
./rustdoc-stripper -s

IMPORTANT: Only files ending with '.rs' will be stripped/regenerated."#
    );
}

fn ask_confirmation(out_file: &str) -> bool {
    let r = io::stdin();
    let mut reader = r.lock();
    let mut line = String::new();
    let stdout = io::stdout();
    let mut stdo = stdout.lock();

    print!(
        r##"A file '{}' already exists. If you want to run rustdoc-stripper anyway, it'll erase the file
and its data. Which means that if your files don't have rustdoc comments anymore, you'll loose them.
Do you want to continue ? (y/n) "##,
        out_file
    );
    let _ = stdo.flush();

    match reader.read_line(&mut line) {
        Ok(_) => {
            line = line.trim().to_owned();
            if line != "y" && line != "Y" {
                if line == "n" || line == "N" {
                    println!("Aborting...");
                } else {
                    println!("Unknown answer: '{}'.\nAborting...", line);
                }
                false
            } else {
                true
            }
        }
        Err(e) => {
            println!("An error occured: {}.\nAborting...", e);
            false
        }
    }
}

fn main() {
    let mut args = ExecOptions {
        stdout_output: false,
        strip: false,
        regenerate: false,
        ignore_macros: false,
        ignore_doc_commented: false,
    };
    let mut first = true;
    let mut wait_filename = false;
    let mut wait_directory = false;
    let mut files_to_ignore = vec![];
    let mut directory = ".".to_owned();
    let mut verbose = false;
    let mut force = false;
    let mut wait_out_file = false;
    let mut out_file = OUTPUT_COMMENT_FILE.to_owned();
    let mut file = None;

    for argument in env::args() {
        if first {
            first = false;
            continue;
        }
        if wait_filename {
            files_to_ignore.push(argument.clone());
            wait_filename = false;
            continue;
        }
        if wait_directory {
            directory = argument.clone();
            wait_directory = false;
            continue;
        }
        if wait_out_file {
            out_file = argument.clone();
            wait_out_file = false;
            continue;
        }
        match &*argument {
            "-h" | "--help" => {
                print_help();
                return;
            }
            "-s" | "--strip" => {
                if !check_options(&mut args, 's') {
                    return;
                }
            }
            "-i" | "--ignore" => {
                wait_filename = true;
            }
            "-d" | "--dir" => {
                wait_directory = true;
            }
            "-o" | "--comment-file" => {
                wait_out_file = true;
            }
            "-g" | "--regenerate" => {
                if !check_options(&mut args, 'g') {
                    return;
                }
            }
            "-n" | "--no-file-output" => {
                args.stdout_output = true;
            }
            "-v" | "--verbose" => {
                verbose = true;
            }
            "-f" | "--force" => {
                force = true;
            }
            "-m" | "--ignore-macros" => {
                args.ignore_macros = true;
            }
            "-x" | "--ignore-doc-commented" => {
                args.ignore_doc_commented = true;
            }
            "-" | "--" => {
                println!("Unknown option: '-'");
                return;
            }
            s => {
                if !s.starts_with('-') {
                    file = Some(s.to_string());
                    continue;
                }
                for c in (&s[1..]).chars() {
                    match c {
                        's' | 'g' => {
                            if !check_options(&mut args, c) {
                                return;
                            }
                        }
                        'n' => {
                            args.stdout_output = true;
                        }
                        'm' => {
                            args.ignore_macros = true;
                        }
                        'x' => {
                            args.ignore_doc_commented = true;
                        }
                        'h' => {
                            print_help();
                            return;
                        }
                        'v' => {
                            verbose = true;
                        }
                        'f' => {
                            force = true;
                        }
                        err if err == 'i' || err == 'd' => {
                            println!(
                                "'{}' have to be used separately from other options. Example:",
                                err
                            );
                            println!("./doc-stripper -s -{} foo", err);
                            return;
                        }
                        err => {
                            println!("Unknown option: {}", err);
                            return;
                        }
                    }
                }
            }
        }
    }
    if wait_filename {
        println!("[-i | --ignore] option expects a filename. Example:");
        println!("./doc-stripper -i src/foo.rs");
        return;
    }
    if wait_directory {
        println!("[-d | --dir] option expects a directory path. Example:");
        println!("./doc-stripper -d src/");
        return;
    }
    if wait_out_file {
        println!("[-o | --comment-file] option expects a file path. Example:");
        println!("./doc-stripper -o src/out.md");
        return;
    }

    let file = match file {
        Some(f) => f,
        None => {
            eprintln!("expected an entry file, found nothing");
            return;
        }
    };

    eprintln!("starting with {}", file);

    if !args.regenerate || args.strip {
        let comments_path = Path::new(&out_file);

        if comments_path.exists() {
            if comments_path.is_file() {
                if !force && !ask_confirmation(&out_file) {
                    return;
                }
            } else {
                println!(
                    "An element called '{}' already exists. Aborting...",
                    &out_file
                );
                return;
            }
        }
        println!("Starting stripping...");
        if args.stdout_output {
            let stdout = io::stdout();
            let mut stdout = stdout.lock();
            if let Err(e) = strip_comments(&file, &mut stdout, args.ignore_macros) {
                eprintln!("doc-stripper failed: {:?}", e);
            }
        } else {
            match File::create(&out_file) {
                Ok(mut f) => {
                    if let Err(e) = strip_comments(&file, &mut f, args.ignore_macros) {
                        eprintln!("doc-stripper failed: {:?}", e);
                    }
                }
                Err(e) => {
                    println!("Error while opening \"{}\": {}", &out_file, e);
                    return;
                }
            }
        }
    } else {
        println!("Starting regeneration...");
        // regenerate_doc_comments(
        //     &directory,
        //     verbose,
        //     &out_file,
        //     args.ignore_macros,
        //     args.ignore_doc_commented,
        // );
    }
    println!("Done !");
}
