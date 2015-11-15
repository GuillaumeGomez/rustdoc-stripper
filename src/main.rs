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

use std::{env, io};
use std::fs::OpenOptions;
use strip::loop_over_files;

mod regenerate;
mod strip;
mod types;

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
                        's' | 'c' => {
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
        println!("{}", argument);
    }
    println!("Starting...");
    if args.stdout_output {
        let tmp = io::stdout();

        loop_over_files(".", &mut tmp.lock());
    } else if args.strip == true || (args.strip == false && args.regenerate == false) {
        match OpenOptions::new().write(true).create(true).truncate(true).open("comments.cmts") {
            Ok(mut f) => {
                loop_over_files(".", &mut f);
                /*for com_entry in comments {
                    write!(f, "{}", com_entry).unwrap();
                }*/
            }
            Err(e) => {
                println!("Error while opening \"{}\": {}", "comments.cmts", e);
                return;
            }
        }
    } else {
        println!("Not implemented yet");
        return;
    }
    println!("Done !");
}
