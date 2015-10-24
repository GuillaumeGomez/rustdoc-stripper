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
    args: Vec<String>,
}

impl TypeStruct {
    fn new(ty: Type, name: &str) -> TypeStruct {
        TypeStruct {
            ty: ty,
            name: name.to_owned(),
            args: vec!(),
            parent: None,
        }
    }

    fn from_args(ty: Type, args: Vec<String>) -> TypeStruct {
        TypeStruct {
            ty: ty,
            name: String::new(),
            args: args,
            parent: None,
        }
    }

    fn copy(&self) -> TypeStruct {
        TypeStruct {
            ty: self.ty,
            name: self.name.clone(),
            args: self.args.clone(),
            parent: None,
        }
    }
}

impl Display for TypeStruct {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let parent = &self.parent;
        match parent {
            &Some(ref p) => writeln!(f, "{}{} {}{}", p, self.ty, self.name, self.args.join(" ")),
            &None => writeln!(f, "{} {}{}", self.ty, self.name, self.args.join(" ")),
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum Type {
    Struct,
    Mod,
    Enum,
    Fn,
    Const,
    Static,
    Type,
    Variant,
    Impl,
    Unknown,
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
            "impl" => Type::Impl,
            _ => Type::Variant,
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match *self {
            Type::Struct => write!(f, "struct"),
            Type::Mod => write!(f, "mod"),
            Type::Enum => write!(f, "enum"),
            Type::Fn => write!(f, "fn"),
            Type::Const => write!(f, "const"),
            Type::Static => write!(f, "static"),
            Type::Type => write!(f, "type"),
            Type::Variant => write!(f, "variant"),
            Type::Impl => write!(f, "impl"),
            _ => write!(f, "?"),
        }
    }
}

fn loop_over_files(path: &str, f: &mut File) {
    match fs::read_dir(path) {
        Ok(it) => {
            for entry in it {
                check_path_type(entry.unwrap().path().to_str().unwrap(), f);
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

fn get_impl(words: &[&str], it: &mut usize) -> Vec<String> {
    let mut v = vec!();

    while (*it + 1) < words.len() {
        if words[(*it + 1)] == "{" || words[(*it + 1)] == ";" {
            break;
        }
        *it += 1;
        v.push(words[*it].to_owned());
    }
    v
}

fn add_to_type_scope(current: &Option<TypeStruct>, e: &Option<TypeStruct>) -> Option<TypeStruct> {
    match current {
        &Some(ref c) => {
            match e {
                &Some(ref t) => {
                    let mut tmp = t.copy();
                    tmp.parent = Some(Box::new(c.copy()));
                    Some(tmp)
                }
                &None => Some(c.copy()),
            }
        },
        &None => match e {
            &Some(ref t) => Some(t.copy()),
            &None => None,
        }
    }
}

fn type_out_scope(current: &Option<TypeStruct>) -> Option<TypeStruct> {
    match current {
        &Some(ref c) => match c.parent {
            Some(ref p) => Some(p.copy()),
            None => None,
        },
        &None => None,
    }
}

fn strip_comments(path: &str, out_file: &mut File) {
    match File::open(path) {
        Ok(mut f) => {
            let mut b_content = String::new();
            f.read_to_string(&mut b_content).unwrap();
            let content = b_content.replace("{", " { ")
                                   .replace("}", " } ")
                                   .replace("///", "/// ")
                                   .replace("//!", "//! ")
                                   .replace("\n", " \n ")
                                   .replace("(", " (");
            let b_content : Vec<&str> = b_content.split('\n').collect();
            let words : Vec<&str> = content.split(' ').filter(|s| s.len() > 0).collect();
            let mut it = 0;
            let mut line = 0;
            let mut event_list = vec!();

            while it < words.len() {
                match words[it] {
                    "///" | "//!" => {
                        event_list.push(EventType::Comment(b_content[line].to_owned()));
                        move_to(&words, &mut it, "\n");
                    }
                    "struct" | "mod" | "fn" | "enum" | "const" | "static" | "type" => {
                        event_list.push(EventType::Type(TypeStruct::new(Type::from(words[it]), words[it + 1])));
                        it += 1;
                    }
                    "impl" => {
                        event_list.push(EventType::Type(TypeStruct::from_args(Type::Impl, get_impl(&words, &mut it))));
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
                    _ => {
                        event_list.push(EventType::Type(TypeStruct::new(Type::Unknown, words[it])));
                    }
                }
                it += 1;
            }
            writeln!(out_file, "=! {}", path).unwrap();
            let mut current : Option<TypeStruct> = None;
            let mut waiting_type : Option<TypeStruct> = None;

            it = 0;
            while it < event_list.len() {
                match event_list[it] {
                    EventType::Type(ref t) => {
                        waiting_type = Some(t.copy());
                    }
                    EventType::InScope => {
                        current = add_to_type_scope(&current, &waiting_type);
                        waiting_type = None;
                    }
                    EventType::OutScope => {
                        current = type_out_scope(&current);
                        waiting_type = None;
                    }
                    EventType::Comment(ref c) => {
                        let mut comments = format!("{}\n", c);

                        it += 1;
                        while match event_list[it] {
                            EventType::Comment(ref c) => {
                                comments.push_str(&format!("{}\n", c));
                                true
                            }
                            EventType::Type(ref t) => {
                                write!(out_file, "=|{}{}", t, comments).unwrap();
                                false
                            }
                            _ => panic!("Comments cannot be written everywhere"),
                        } {
                            it += 1;
                        }
                        it -= 1;
                    }
                }
                it += 1;
            }
        }
        Err(e) => {
            println!("Unable to open \"{}\": {}", path, e);
        }
    }
}

fn check_path_type(path: &str, f: &mut File) {
    match fs::metadata(path) {
        Ok(m) => {
            if m.is_dir() {
                if path == ".." || path == "." {
                    return;
                }
                loop_over_files(path, f);
            } else {
                if path == "./comments.cmts" {
                    return;
                }
                println!("-> {}", path);
                strip_comments(path, f);
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
    match OpenOptions::new().write(true).create(true).truncate(true).open("comments.cmts") {
        Ok(mut f) => {
            loop_over_files(".", &mut f);
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