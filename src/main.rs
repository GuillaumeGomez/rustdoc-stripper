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
use std::io::{BufRead, Write, Read};
use std::fmt::{Display, Formatter, Error};
use std::ops::Deref;

enum EventType {
    Comment(String),
    FileComment(String),
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

    fn empty() -> TypeStruct {
        TypeStruct {
            ty: Type::Unknown,
            name: String::new(),
            args: Vec::new(),
            parent: None,
        }
    }
}

impl Clone for TypeStruct {
    fn clone(&self) -> TypeStruct {
        TypeStruct {
            ty: self.ty,
            name: self.name.clone(),
            args: self.args.clone(),
            parent: match self.parent {
                Some(ref p) => Some(Box::new(p.deref().clone())),
                None => None,
            }
        }
    }

    fn clone_from(&mut self, source: &TypeStruct) {
        self.ty = source.ty;
        self.name = source.name.clone();
        self.args = source.args.clone();
        self.parent = match source.parent {
            Some(ref p) => Some(Box::new(p.deref().clone())),
            None => None,
        };
    }
}

impl Display for TypeStruct {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let parent = &self.parent;
        match parent {
            &Some(ref p) => write!(f, "{}({} {}{})", p, self.ty, self.name, self.args.join(" ")),
            &None => write!(f, "{} {}{}", self.ty, self.name, self.args.join(" ")),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
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
    Use,
    Macro,
    Trait,
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
            "use" => Type::Use,
            "trait" => Type::Trait,
            "macro" => Type::Macro,
            "macro_rules" => Type::Macro,
            "macro_rules!" => Type::Macro,
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
            Type::Use => write!(f, "use"),
            Type::Trait => write!(f, "trait"),
            Type::Macro => write!(f, "macro"),
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

fn move_to(words: &[&str], it: &mut usize, limit: &str, line: &mut usize) {
    while (*it + 1) < words.len() && limit.contains(words[*it + 1]) == false {
        if words[*it] == "\n" {
            *line += 1;
        }
        *it += 1;
    }
    if words[*it] == "\n" {
        *line += 1;
    }
}

fn move_until(words: &[&str], it: &mut usize, limit: &str, line: &mut usize) {
    while (*it + 1) < words.len() && limit != words[*it] {
        if words[*it] == "\n" {
            *line += 1;
        }
        *it += 1;
    }
}

fn get_impl(words: &[&str], it: &mut usize, line: &mut usize) -> Vec<String> {
    let mut v = vec!();

    while (*it + 1) < words.len() {
        if words[*it] == "\n" {
            *line += 1;
        }
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
                    let mut tmp = t.clone();
                    tmp.parent = Some(Box::new(c.clone()));
                    Some(tmp)
                }
                &None => {
                    let mut tmp = TypeStruct::empty();
                    tmp.parent = Some(Box::new(c.clone()));
                    Some(tmp)
                }
            }
        },
        &None => match e {
            &Some(ref t) => Some(t.clone()),
            &None => None,
        }
    }
}

fn type_out_scope(current: &Option<TypeStruct>) -> Option<TypeStruct> {
    match current {
        &Some(ref c) => match c.parent {
            Some(ref p) => Some(p.deref().clone()),
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
                                   .replace("/*!", "/*! ")
                                   .replace("*/", " */")
                                   .replace("\n", " \n ")
                                   .replace("!(", " !! (")
                                   .replace(",", ", ")
                                   .replace("(", " (");
            let b_content : Vec<&str> = b_content.split('\n').collect();
            let words : Vec<&str> = content.split(' ').filter(|s| s.len() > 0).collect();
            let mut it = 0;
            let mut line = 0;
            let mut event_list = vec!();
            let mut comments = 0;

            while it < words.len() {
                match words[it] {
                    "///" => {
                        event_list.push(EventType::Comment(b_content[line].to_owned()));
                        move_to(&words, &mut it, "\n", &mut line);
                        comments += 1;
                    }
                    "//!" => {
                        event_list.push(EventType::FileComment(b_content[line].to_owned()));
                        move_to(&words, &mut it, "\n", &mut line);
                        comments += 1;
                    }
                    "/*!" => {
                        let mark = line;
                        move_until(&words, &mut it, "*/", &mut line);
                        for pos in mark..line {
                            event_list.push(EventType::FileComment(b_content[pos].to_owned()));
                        }
                        event_list.push(EventType::FileComment("*/".to_owned()));
                        comments += 1;
                    }
                    "struct" | "mod" | "fn" | "enum" | "const" | "static" | "type" | "use" | "trait" | "macro_rules!" => {
                        event_list.push(EventType::Type(TypeStruct::new(Type::from(words[it]), words[it + 1])));
                        it += 1;
                    }
                    "!!" => {
                        event_list.push(EventType::Type(TypeStruct::new(Type::from("macro"), &format!("{}!{}", words[it - 1], words[it + 1]))));
                        it += 1;
                    }
                    "impl" => {
                        event_list.push(EventType::Type(TypeStruct::from_args(Type::Impl, get_impl(&words, &mut it, &mut line))));
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
            if comments < 1 {
                return;
            }
            writeln!(out_file, "=! {}", path).unwrap();
            let mut current : Option<TypeStruct> = None;
            let mut waiting_type : Option<TypeStruct> = None;

            it = 0;
            while it < event_list.len() {
                match event_list[it] {
                    EventType::Type(ref t) => {
                        if t.ty != Type::Unknown {
                            waiting_type = Some(t.clone());
                            //println!("current : {}", t);
                        }
                    }
                    EventType::InScope => {
                        current = add_to_type_scope(&current, &waiting_type);
                        /*if waiting_type.is_some() {
                            println!("in : {}", waiting_type.unwrap());
                        }*/
                        waiting_type = None;
                    }
                    EventType::OutScope => {
                        /*if current.is_some() {
                            println!("out : {}", current.clone().unwrap());
                        }*/
                        current = type_out_scope(&current);
                        waiting_type = None;
                    }
                    EventType::FileComment(ref c) => {
                        let mut comments = format!("=/ {}\n", c);

                        it += 1;
                        while match event_list[it] {
                            EventType::FileComment(ref c) => {
                                comments.push_str(&format!("=/ {}\n", c));
                                true
                            }
                            _ => false,
                        } {
                            it += 1;
                        }
                        write!(out_file, "{}", comments).unwrap();
                    }
                    EventType::Comment(ref c) => {
                        let mut comments = format!("{}\n", c);

                        it += 1;
                        while match event_list[it] {
                            EventType::Comment(ref c) => {
                                comments.push_str(&format!("{}\n", c));
                                true
                            }
                            EventType::Type(_) => {
                                false
                            }
                            _ => panic!("Comments cannot be written everywhere"),
                        } {
                            it += 1;
                        }
                        while match event_list[it] {
                            EventType::Type(ref t) => {
                                match t.ty {
                                    Type::Unknown => {
                                        match current {
                                            Some(ref cur) => {
                                                if cur.ty == Type::Enum || cur.ty == Type::Struct || cur.ty == Type::Use {
                                                    if t.name == "pub" {
                                                        true
                                                    } else {
                                                        let mut copy = t.clone();
                                                        copy.ty = Type::Variant;
                                                        let tmp = add_to_type_scope(&current, &Some(copy));
                                                        write!(out_file, "=| {}\n{}", tmp.unwrap(), comments).unwrap();
                                                        false
                                                    }
                                                } else {
                                                    false
                                                }
                                            }
                                            None => false,
                                        }
                                    },
                                    _ => {
                                        let tmp = add_to_type_scope(&current, &Some(t.clone()));
                                        write!(out_file, "=| {}\n{}", tmp.unwrap(), comments).unwrap();
                                        false
                                    }
                                }
                            }
                            _ => panic!("An item was expected for this comment: {}", comments),
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