// Copyright 2015 Gomez Guillaume
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate stripper_interface;

use regenerate::regenerate_doc_comments;
use std::{env, io};
use std::io::{BufRead, Write};
use std::fs::OpenOptions;
use std::path::Path;
use strip::strip_comments;
use types::OUTPUT_COMMENT_FILE;
use utils::loop_over_files;

mod regenerate;
mod strip;
mod types;
mod utils;

struct ExecOptions {
    stdout_output: bool,
    strip: bool,
    regenerate: bool,
}

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
    println!(r#"Available options for rustdoc-stripper:
    -h | --help             : Displays this help
    -s | --strip            : Strips the specified folder's files and create a file
                              with rustdoc information (comments.cmts by default)
    -g | --regenerate       : Recreate files with rustdoc comments from reading
                              rustdoc information file (comments.cmts by default)
    -n | --no-file-output   : Display rustdoc information directly on stdout
    -i | --ignore [filename]: Ignore the specified file, can be repeated as much
                              as needed, only used when stripping files, ignored
                              otherwise
    -d | --dir [directory]  : Specify a directory path to work on, optional
    -v | --verbose          : Activate verbose mode
    -f | --force            : Remove confirmation demands

By default, rustdoc is run with -s option:
./rustdoc-stripper -s

IMPORTANT: Only files ending with '.rs' will be stripped/regenerated."#);
}

fn ask_confirmation() -> bool {
    let r = io::stdin();
    let mut reader = r.lock();
    let mut line = String::new();
    let stdout = io::stdout();
    let mut stdo = stdout.lock();

    print!(r##"A file '{}' already exists. If you want to run rustdoc-stripper anyway, it'll erase the file
and its data. Which means that if your files don't have rustdoc comments anymore, you'll loose them.
Do you want to continue ? (y/n) "##, OUTPUT_COMMENT_FILE);
    stdo.flush();

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
    };
    let mut first = true;
    let mut wait_filename = false;
    let mut wait_directory = false;
    let mut files_to_ignore = vec!();
    let mut directory = ".".to_owned();
    let mut verbose = false;
    let mut force = false;

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
            s => {
                if s.chars().next().unwrap() != '-' {
                    println!("Unknown option: {}", s);
                    return;
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
                            println!("'{}' have to be used separately from other options. Example:", err);
                            println!("./rustdoc-stripper -s -{} foo", err);
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
        println!("-i option expects a filename. Example:");
        println!("./rustdoc-stripper -i src/foo.rs");
        return;
    }
    if wait_directory {
        println!("-d option expects a directory path. Example:");
        println!("./rustdoc-stripper -d src/");
        return;
    }

    if args.strip == true || (args.strip == false && args.regenerate == false) {
        let comments_path = Path::new(OUTPUT_COMMENT_FILE);

        if comments_path.exists() {
            if comments_path.is_file() {
                if !force && !ask_confirmation() {
                    return;
                }
            } else {
                println!("An element called '{}' already exist. Aborting...", OUTPUT_COMMENT_FILE);
                return;
            }
        }
        println!("Starting stripping...");
        if args.stdout_output {
            let tmp = io::stdout();

            loop_over_files(&directory, &mut tmp.lock(), &strip_comments, &files_to_ignore, verbose);
        } else {
            match OpenOptions::new().write(true).create(true).truncate(true).open(OUTPUT_COMMENT_FILE) {
                Ok(mut f) => {
                    loop_over_files(&directory, &mut f, &strip_comments, &files_to_ignore, verbose);
                }
                Err(e) => {
                    println!("Error while opening \"{}\": {}", OUTPUT_COMMENT_FILE, e);
                    return;
                }
            }
        }
    } else {
        println!("Starting regeneration...");
        regenerate_doc_comments(&directory, verbose);
        return;
    }
    println!("Done !");
}
