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
use std::process::exit;
use std::ops::Deref;

static MOD_COMMENT : &'static str = "=|";
static FILE_COMMENT : &'static str = "=/";
static FILE : &'static str = "=!";

use types::{
    TypeStruct,
    EventType,
    Type,
};

pub fn loop_over_files<F: Write>(path: &str, f: &mut F) {
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

fn get_mod<F: Write>(current: &Option<TypeStruct>, out_file: &mut F) -> bool {
    match *current {
        Some(ref t) => {
            if t.ty != Type::Mod {
                println!("Mod/File comments cannot be put here!");
                false
            } else {
                writeln!(out_file, "{} {}", MOD_COMMENT, t).unwrap();
                true
            }
        }
        None => true,
    }
}

fn strip_comments<F: Write>(path: &str, out_file: &mut F) {
    match File::open(path) {
        Ok(mut f) => {
            let mut b_content = String::new();
            f.read_to_string(&mut b_content).unwrap();
            let content = b_content.replace("{", " { ")
                                   .replace("}", " } ")
                                   .replace("///", "/// ")
                                   .replace("//!", "//! ")
                                   .replace("/*!", "/*! ")
                                   .replace(":", " : ")
                                   .replace("*/", " */")
                                   .replace("\n", " \n ")
                                   .replace("!(", " !! (")
                                   .replace(",", ", ")
                                   .replace("(", " (");
            let mut b_content : Vec<&str> = b_content.split('\n').collect();
            let words : Vec<&str> = content.split(' ').filter(|s| s.len() > 0).collect();
            let mut it = 0;
            let mut line = 0;
            let mut event_list = vec!();
            let mut comments = 0;
            let mut to_remove = vec!();

            while it < words.len() {
                match words[it] {
                    "///" => {
                        to_remove.push(line);
                        event_list.push(EventType::Comment(b_content[line].to_owned()));
                        move_to(&words, &mut it, "\n", &mut line);
                        comments += 1;
                    }
                    "//!" => {
                        to_remove.push(line);
                        event_list.push(EventType::FileComment(b_content[line].to_owned()));
                        move_to(&words, &mut it, "\n", &mut line);
                        comments += 1;
                    }
                    "/*!" => {
                        let mark = line;
                        move_until(&words, &mut it, "*/", &mut line);
                        for pos in mark..line {
                            to_remove.push(mark + pos);
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
            writeln!(out_file, "{} {}", FILE, path).unwrap();
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
                        // first, we need to find if it belongs to a mod
                        if get_mod(&current, out_file) == false {
                            exit(1);
                        }
                        it += 1;
                        let mut comments = format!("{} {}\n", FILE_COMMENT, c);
                        while match event_list[it] {
                            EventType::FileComment(ref c) => {
                                comments.push_str(&format!("{} {}\n", FILE_COMMENT, c));
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
                            _ => panic!("Doc comments cannot be written everywhere"),
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
                                                        write!(out_file, "{} {}\n{}", MOD_COMMENT, tmp.unwrap(), comments).unwrap();
                                                        false
                                                    }
                                                } else {
                                                    if t.name == "pub" {
                                                        true
                                                    } else {
                                                        false
                                                    }
                                                }
                                            }
                                            None => false,
                                        }
                                    },
                                    _ => {
                                        let tmp = add_to_type_scope(&current, &Some(t.clone()));
                                        write!(out_file, "{} {}\n{}", MOD_COMMENT, tmp.unwrap(), comments).unwrap();
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
            // we now remove doc comments from original file
            remove_comments(path, &to_remove, &mut b_content);
        }
        Err(e) => {
            println!("Unable to open \"{}\": {}", path, e);
        }
    }
}

fn remove_comments(path: &str, to_remove: &[usize], o_content: &mut Vec<&str>) {
    match OpenOptions::new().write(true).create(true).truncate(true).open(format!("{}_", path)) {
        Ok(mut f) => {
            let mut decal = 0;

            for line in to_remove {
                o_content.remove(line - decal);
                decal += 1;
            }
            write!(f, "{}", o_content.join("\n")).unwrap();
        }
        Err(e) => {
            println!("Cannot open {}_: {}", path, e);
        }
    }
}

fn check_path_type<F: Write>(path: &str, f: &mut F) {
    match fs::metadata(path) {
        Ok(m) => {
            if m.is_dir() {
                if path == ".." || path == "." {
                    return;
                }
                loop_over_files(path, f);
            } else {
                if path == "./comments.cmts" || !path.ends_with(".rs") {
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