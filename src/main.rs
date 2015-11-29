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
    -h | --help          : Displays this help
    -s | --strip         : Strips the current folder files and create a file
                           with rustdoc information (comments.cmts by default)
    -g | --regenerate    : Recreate files with rustdoc comments from reading
                           rustdoc information file (comments.cmts by default)
    -n | --no-file-output: Display rustdoc information directly on stdout

By default, rustdoc is run with -s option:
./rustdoc-stripper -s"#);
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

    for argument in env::args() {
        if first {
            first = false;
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
            "-g" | "--regenerate" => {
                if !check_options(&mut args, 'g') {
                    return;
                }
            }
            "-n" | "--no-file-output" => {
                args.stdout_output = true;
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
                        err => {
                            println!("Unknown option: {}", err);
                            return;
                        }
                    }
                }
            }
        }
    }

    if args.strip == true || (args.strip == false && args.regenerate == false) {
        let comments_path = Path::new(OUTPUT_COMMENT_FILE);

        if comments_path.exists() {
            if comments_path.is_file() {
                if !ask_confirmation() {
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

            loop_over_files(".", &mut tmp.lock(), &strip_comments);
        } else {
            match OpenOptions::new().write(true).create(true).truncate(true).open(OUTPUT_COMMENT_FILE) {
                Ok(mut f) => {
                    loop_over_files(".", &mut f, &strip_comments);
                }
                Err(e) => {
                    println!("Error while opening \"{}\": {}", OUTPUT_COMMENT_FILE, e);
                    return;
                }
            }
        }
    } else {
        println!("Starting regeneration...");
        regenerate_doc_comments();
        return;
    }
    println!("Done !");
}
