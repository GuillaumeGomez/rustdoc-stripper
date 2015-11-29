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

use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, Write, Read};
use std::process::exit;
use std::ops::Deref;
use utils::join;

use types::{
    EventInfo,
    EventType,
    ParseResult,
    TypeStruct,
    Type,
    MOD_COMMENT,
    FILE_COMMENT,
    FILE,
};

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

pub fn add_to_type_scope(current: &Option<TypeStruct>, e: &Option<TypeStruct>) -> Option<TypeStruct> {
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

pub fn type_out_scope(current: &Option<TypeStruct>) -> Option<TypeStruct> {
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
                writeln!(out_file, "{}{}", MOD_COMMENT, t).unwrap();
                true
            }
        }
        None => true,
    }
}

pub fn build_event_list(path: &str) -> io::Result<ParseResult> {
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
                                   .replace(" :  : ", "::")
                                   .replace("*/", " */")
                                   .replace("\n", " \n ")
                                   .replace("!(", " !! (")
                                   .replace(",", ", ")
                                   .replace("(", " (");
            let b_content : Vec<String> = b_content.split('\n').map(|s| s.to_owned()).collect();
            let words : Vec<&str> = content.split(' ').filter(|s| s.len() > 0).collect();
            let mut it = 0;
            let mut line = 0;
            let mut event_list = vec!();
            let mut comment_lines = vec!();

            while it < words.len() {
                match words[it] {
                    "///" => {
                        comment_lines.push(line);
                        event_list.push(EventInfo::new(line, EventType::Comment(b_content[line].to_owned())));
                        move_to(&words, &mut it, "\n", &mut line);
                    }
                    "//!" => {
                        comment_lines.push(line);
                        event_list.push(EventInfo::new(line, EventType::FileComment(b_content[line].to_owned())));
                        if line + 1 < b_content.len() && b_content[line + 1].len() < 1 {
                            comment_lines.push(line + 1);
                            event_list.push(EventInfo::new(line, EventType::FileComment("".to_owned())));
                        }
                        move_to(&words, &mut it, "\n", &mut line);
                    }
                    "/*!" => {
                        let mark = line;
                        move_until(&words, &mut it, "*/", &mut line);
                        for pos in mark..line {
                            comment_lines.push(pos);
                            event_list.push(EventInfo::new(line, EventType::FileComment(b_content[pos].to_owned())));
                        }
                        comment_lines.push(line);
                        let mut removed = false;
                        if line + 1 < b_content.len() && b_content[line + 1].len() < 1 {
                            comment_lines.push(line + 1);
                            removed = true;
                        }
                        event_list.push(EventInfo::new(line, EventType::FileComment("*/".to_owned())));
                        if removed {
                            event_list.push(EventInfo::new(line, EventType::FileComment("".to_owned())));
                        }
                    }
                    "struct" | "mod" | "fn" | "enum" | "const" | "static" | "type" | "use" | "trait" | "macro_rules!" => {
                        event_list.push(EventInfo::new(line, EventType::Type(TypeStruct::new(Type::from(words[it]), words[it + 1]))));
                        it += 1;
                    }
                    "!!" => {
                        event_list.push(EventInfo::new(line,
                            EventType::Type(TypeStruct::new(Type::from("macro"), &format!("{}!{}", words[it - 1], words[it + 1])))));
                        it += 1;
                    }
                    "impl" => {
                        event_list.push(EventInfo::new(line,
                            EventType::Type(TypeStruct::new(Type::Impl, &join(&get_impl(&words, &mut it, &mut line), " ")))));
                    }
                    "{" => {
                        event_list.push(EventInfo::new(line, EventType::InScope));
                    }
                    "}" => {
                        event_list.push(EventInfo::new(line, EventType::OutScope));
                    }
                    "\n" => {
                        line += 1;
                    }
                    s if s.starts_with("#[") || s.starts_with("#![") => {
                        while words[it + 1] != "\n" {
                            it += 1;
                        }
                    }
                    _ => {
                        event_list.push(EventInfo::new(line, EventType::Type(TypeStruct::new(Type::Unknown, words[it]))));
                    }
                }
                it += 1;
            }
            Ok(ParseResult { event_list : event_list,
                             comment_lines : comment_lines,
                             original_content : b_content,
                           })
        },
        Err(e) => Err(e),
    }
}

pub fn strip_comments<F: Write>(path: &str, out_file: &mut F) {
    match build_event_list(path) {
        Ok(parse_result) => {
            if parse_result.comment_lines.len() < 1 {
                return;
            }
            writeln!(out_file, "{}{}", FILE, path).unwrap();
            let mut current : Option<TypeStruct> = None;
            let mut waiting_type : Option<TypeStruct> = None;
            let mut it = 0;

            while it < parse_result.event_list.len() {
                match parse_result.event_list[it].event {
                    EventType::Type(ref t) => {
                        if t.ty != Type::Unknown {
                            waiting_type = Some(t.clone());
                        }
                    }
                    EventType::InScope => {
                        current = add_to_type_scope(&current, &waiting_type);
                        waiting_type = None;
                    }
                    EventType::OutScope => {
                        current = type_out_scope(&current);
                        waiting_type = None;
                    }
                    EventType::FileComment(ref c) => {
                        // first, we need to find if it belongs to a mod
                        if get_mod(&current, out_file) == false {
                            exit(1);
                        }
                        it += 1;
                        let mut comments = format!("{}{}\n", FILE_COMMENT, c);
                        while match parse_result.event_list[it].event {
                            EventType::FileComment(ref c) => {
                                comments.push_str(&format!("{}{}\n", FILE_COMMENT, c));
                                true
                            }
                            _ => false,
                        } {
                            it += 1;
                        }
                        write!(out_file, "{}", comments).unwrap();
                        continue;
                    }
                    EventType::Comment(ref c) => {
                        let mut comments = format!("{}\n", c);

                        it += 1;
                        while it < parse_result.event_list.len() &&
                              match parse_result.event_list[it].event {
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
                        if it >= parse_result.event_list.len() {
                            continue;
                        }
                        while match parse_result.event_list[it].event {
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
                                                        write!(out_file, "{}{}\n{}", MOD_COMMENT, tmp.unwrap(), comments).unwrap();
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
                                            None => {
                                                if t.name == "pub" {
                                                    true
                                                } else {
                                                    false
                                                }
                                            },
                                        }
                                    },
                                    _ => {
                                        let tmp = add_to_type_scope(&current, &Some(t.clone()));
                                        write!(out_file, "{}{}\n{}", MOD_COMMENT, tmp.unwrap(), comments).unwrap();
                                        false
                                    }
                                }
                            }
                            _ => panic!("An item was expected for this comment: {}", comments),
                        } {
                            it += 1;
                        }
                        continue;
                    }
                }
                it += 1;
            }
            // we now remove doc comments from original file
            remove_comments(path, &parse_result.comment_lines, parse_result.original_content);
        }
        Err(e) => {
            println!("Unable to open \"{}\": {}", path, e);
        }
    }
}

fn remove_comments(path: &str, to_remove: &[usize], mut o_content: Vec<String>) {
    match OpenOptions::new().write(true).create(true).truncate(true).open(path) {
        Ok(mut f) => {
            let mut decal = 0;

            for line in to_remove {
                o_content.remove(line - decal);
                decal += 1;
            }
            write!(f, "{}", o_content.join("\n")).unwrap();
        }
        Err(e) => {
            println!("Cannot open '{}': {}", path, e);
        }
    }
}
