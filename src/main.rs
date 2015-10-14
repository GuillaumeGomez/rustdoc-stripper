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

use std::{env, fs};
use std::fs::{File, OpenOptions};
use std::io::BufReader;

struct CommentEntry {
    line: String,
    file: String,
    comment_lines: Vec<String>,
}

impl CommentEntry {
    pub fn new(file: &str) -> CommentEntry {
        CommentEntry {
            line: String::new(),
	    file: file.into_owned(),
	    comment_lines: Vec::new(),
        }
    }
}

impl Display for CommentEntry {
    fn display(&self, f: &mut Formatter) -> io::Result<> {
        write!(f, "=|={}", self.line);
        writeln!(f, "=!={}", self.file);
	for comment in self.comment_lines {
             write!(f, "{}", comment);
        }
    }
}

fn loop_over_files(path: &str, comments: &mut Vec<CommentEntry>) {
    match fs::read_dir(path) {
        Ok(it) => {
            for entry in it {
                check_path_type(entry.path().to_str().unwrap(), comments);
            }
        }
        Err(e) => {
            println!("Error while trying to iterate over {}: {}", path, e);
        }
    }
}

fn move_reader(reader: &mut BufReader, first_line: &str, file: &str,
               comments: &mut Vec<CommentEntry>) -> Option<String> {
    let mut cm = CommentEntry::new(file);
    let mut current = String::new();

    cm.comment_lines.push(first_line.to_owned());
    loop {
        if reader.read_line(&mut current).unwrap() < 1 {
            // error
	    return None;
        }
	if current.trim_left().starts_with("///") {
            cm.comment_lines.push(current.clone());
        } else {
            break;
        }
    }
    cm.line = current.clone();
    comments.push(cm);
    Some(current)
}

fn strip_comments(path: &str, comments: &mut Vec<CommentEntry>) {
    match FileOptions::new().read(true).write(true).open(path) {
        Ok(f) => {
            let mut reader = BufReader::new(f);
            let mut line = String::new();
	    let mut out_lines = vec!();

	    while reader.read_line(&mut buffer).unwrap() > 0 {
                let mut worker = buffer.trim_left().to_owned();

                if worker.starts_with("///") {
                    // "normal" doc comments
                    out_lines.push(move_reader(&mut reader, &line, path, comments).unwrap());
                } else if worker.starts_with("//!")/* || worker.starts_with("/*!")*/ {
                    // module comments
                }
                buffer.clear();
            }
	    f.seek(SeekFrom::Start(0));
            let mut writer = BufWriter::new(f);

	    for line in out_lines {
                write!(writer, "{}", line).unwrap();
            }
        }
        Err(e) => {
            println!("Unable to open \"{}\": {}", path, e);
        }
    }
}

fn check_path_type(path: &str, comments: &mut Vec<CommentEntry>) {
    match fs::metadata(path) {
        Ok(m) => {
            if m.is_dir() {
                loop_over_files(path, comments);
            } else {
                strip_comments(path, comments);
            }
        }
        Err(e) => {
            println!("An error occurred: {}", e);
        }
    }
}

fn main() {
    let mut comments = vec!();

    check_path_type(".", &mut comments);
}
