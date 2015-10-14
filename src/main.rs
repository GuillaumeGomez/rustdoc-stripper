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

use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, SeekFrom, BufRead, Write, Read, Seek};
use std::fmt::{Display, Formatter, Error};

struct CommentEntry {
    line: String,
    file: String,
    comment_lines: Vec<String>,
}

impl CommentEntry {
    pub fn new(file: &str) -> CommentEntry {
        CommentEntry {
            line: String::new(),
            file: file.to_owned(),
            comment_lines: Vec::new(),
        }
    }
}

impl Display for CommentEntry {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let mut out = format!("=!={}\n=|={}\n", self.file, self.line);

        //println!("{}: {} => {:?}", self.file, self.line, self.comment_lines);
        for comment in &self.comment_lines {
            out.push_str(&comment);
            out.push_str("\n");
        }
        writeln!(f, "{}", out)
    }
}

fn loop_over_files(path: &str, comments: &mut File) {
    match fs::read_dir(path) {
        Ok(it) => {
            for entry in it {
                check_path_type(entry.unwrap().path().to_str().unwrap(), comments);
            }
        }
        Err(e) => {
            println!("Error while trying to iterate over {}: {}", path, e);
        }
    }
}

fn move_reader(it: &mut usize, lines: &[&str], file: &str,
               comments: &mut File) {
    let mut cm = CommentEntry::new(file);

    cm.comment_lines.push(lines[*it].to_owned());
    *it += 1;
    while *it < lines.len() {
        if lines[*it].trim_left().starts_with("///") {
            cm.comment_lines.push(lines[*it].to_owned());
        } else {
            break;
        }
        *it += 1;
    }
    cm.line = lines[*it].to_owned();
    write!(comments, "{}", cm).unwrap();
    comments.flush();
}

fn strip_comments(path: &str, comments: &mut File) {
    match OpenOptions::new().read(true).write(true).open(path) {
        Ok(mut f) => {
            let mut out_lines = vec!();
            let mut content = String::new();
            f.read_to_string(&mut content).unwrap();
            let lines : Vec<&str> = content.split('\n').collect();
            let mut it = 0;

            while it < lines.len() {
                let worker = lines[it].trim_left().to_owned();

                if worker.starts_with("///") {
                    // "normal" doc comments
                    move_reader(&mut it, &lines, path, comments);
                } else if worker.starts_with("//!")/* || worker.starts_with("/*!*/")*/ {
                    // module comments
                    move_reader(&mut it, &lines, path, comments);
                }
                out_lines.push(lines[it].to_owned());
                it += 1;
            }
            println!("end of loop");
            f.seek(SeekFrom::Start(0)).unwrap();
            for line in out_lines {
                writeln!(f, "{}", line).unwrap();
            }
        }
        Err(e) => {
            println!("Unable to open \"{}\": {}", path, e);
        }
    }
}

fn check_path_type(path: &str, comments: &mut File) {
    match fs::metadata(path) {
        Ok(m) => {
            if m.is_dir() {
                if path == ".." || path == "." {
                    return;
                }
                loop_over_files(path, comments);
            } else {
                println!("-> {}", path);
                strip_comments(path, comments);
            }
        }
        Err(e) => {
            println!("An error occurred: {}", e);
        }
    }
}

fn main() {
    println!("Starting...");
    match OpenOptions::new().write(true).create(true).truncate(true).open("comments.cmts") {
        Ok(mut f) => {
            loop_over_files(".", &mut f);
        }
        Err(e) => {
            println!("Error while opening \"{}\": {}", "comments.cmts", e);
        }
    }
    println!("Done !");
}