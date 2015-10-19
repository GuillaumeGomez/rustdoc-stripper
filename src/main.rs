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

use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, SeekFrom, BufRead, Write, Read, Seek};
use std::fmt::{Display, Formatter, Error};
use std::path::PathBuf;

/*struct CommentEntry {
    file: String,
    text: String,
}

impl CommentEntry {
    fn new(file: &str, text: &str) -> CommentEntry {
        
    }
}

impl Display for CommentEntry {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        //let mut out = format!("=!={}\n=|={}\n", self.file, self.type_);
        let mut out = String::new();

        //println!("{}: {} => {:?}", self.file, self.line, self.comment_lines);
        /*for comment in &self.comment_lines {
            out.push_str(&comment);
            out.push_str("\n");
        }*/
        //writeln!(f, "{}", out)
        writeln!(f, "=!={}\n{}", self.file, self.text)
    }
}*/

enum EventType {
    Comment(String),
    Type(TypeStruct),
    InScope,
    OutScope,
}

struct TypeStruct {
    ty: Type,
    parent: Option<Box<TypeStruct>>,
    name: String,
}

impl TypeStruct {
    fn new(ty: Type, name: &str) -> TypeStruct {
        TypeStruct {
            ty: ty,
            name: name.to_owned(),
            parent: None,
        }
    }
}

#[derive(Debug)]
enum Type {
    Struct,
    Mod,
    Enum,
    Fn,
    Const,
    Static,
    Type,
    Variant,
}

impl Type {
    fn from(s: &str) -> Type {
        match s {
            "struct" => Type::Struct,
            "mod" => Type::Mod,
            "enum" => Type::Enum,
            "fn" => Type::Fn,
            "const" => Type::Const,
            "static" => Type::Static,
            "type" => Type::Type,
            _ => Type::Variant,
        }
    }
}

fn loop_over_files(path: &str/*, comments: &mut Vec<CommentEntry>*/) {
    match fs::read_dir(path) {
        Ok(it) => {
            for entry in it {
                check_path_type(entry.unwrap().path().to_str().unwrap()/*, comments*/);
            }
        }
        Err(e) => {
            println!("Error while trying to iterate over {}: {}", path, e);
        }
    }
}

fn move_to(words: &[&str], it: &mut usize, limit: &str) {
    while (*it + 1) < words.len() && words[*it + 1] != limit {
        *it += 1;
    }
}

fn strip_comments(path: &str/*, comments: &mut Vec<CommentEntry>*/) {
    match File::open(path) {
        Ok(mut f) => {
            let mut b_content = String::new();
            f.read_to_string(&mut b_content).unwrap();
            let content = b_content.replace("{", " { ")
                                   .replace("}", " } ")
                                   .replace("///", "/// ")
                                   .replace("\n", " \n ")
                                   .replace("(", " (");
            let b_content : Vec<&str> = b_content.split('\n').collect();
            let words : Vec<&str> = content.split(' ').filter(|s| s.len() > 0).collect();
            let mut it = 0;
            let mut line = 0;
            let mut event_list = vec!();

            while it < words.len() {
                match words[it] {
                    "///" => {
                        event_list.push(EventType::Comment(b_content[line].to_owned()));
                        move_to(&words, &mut it, "\n");
                    }
                    "struct" | "mod" | "fn" | "enum" | "const" | "static" | "type" => {
                        event_list.push(EventType::Type(TypeStruct::new(Type::from(words[it]), words[it + 1])));
                        it += 1;
                    }
                    "{" => {
                        event_list.push(EventType::InScope);
                    }
                    "}" => {
                        event_list.push(EventType::OutScope);
                    }
                    "\n" => {
                        line += 1;
                    }
                    _ => {}
                }
                it += 1;
            }
            for event in event_list {
                match event {
                    EventType::Comment(s) => { println!("{}", s); }
                    EventType::Type(t) => { println!("{:?} {}", t.ty, t.name);}
                    EventType::InScope => { println!("{{"); }
                    EventType::OutScope => { println!("}}"); }
                }
            }
        }
        Err(e) => {
            println!("Unable to open \"{}\": {}", path, e);
        }
    }
}

fn check_path_type(path: &str/*, comments: &mut Vec<CommentEntry>*/) {
    match fs::metadata(path) {
        Ok(m) => {
            if m.is_dir() {
                if path == ".." || path == "." {
                    return;
                }
                loop_over_files(path/*, comments*/);
            } else {
                if path == "./comments.cmts" {
                    return;
                }
                println!("-> {}", path);
                strip_comments(path/*, comments*/);
            }
        }
        Err(e) => {
            println!("An error occurred: {}", e);
        }
    }
}

fn main() {
    println!("Starting...");
    match fs::remove_file("comments.cmts") { _ => {} }
    //let mut comments = vec!();
    loop_over_files("."/*, &mut comments*/);
    match OpenOptions::new().write(true).create(true).truncate(true).open("comments.cmts") {
        Ok(mut f) => {
            /*for com_entry in comments {
                write!(f, "{}", com_entry).unwrap();
            }*/
            println!("Done !");
        }
        Err(e) => {
            println!("Error while opening \"{}\": {}", "comments.cmts", e);
        }
    }
}

// au lieu de splitter le fichier via '\n', lis le sur "une seule ligne".
// quand tu croises un "///", tu stockes ça comme un commentaire, un "mod", comme un module, etc...
// il faudra compter les '{' / '}' pour voir si t'es toujours danns le truc courant ou non