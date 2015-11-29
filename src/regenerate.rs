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

use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::collections::HashMap;
use strip;
use utils::loop_over_files;

use types::{
    TypeStruct,
    EventType,
    Type,
    MOD_COMMENT,
    FILE_COMMENT,
    FILE,
};

fn get_corresponding_type(elements: &[(Option<TypeStruct>, Vec<String>)],
                          to_find: &Option<TypeStruct>,
                          mut line: usize,
                          decal: &mut usize,
                          original_content: &mut Vec<String>) -> Option<usize> {
    let mut pos = 0;

    while pos < elements.len() {
        if match (&elements[pos].0, to_find) {
            (&Some(ref a), &Some(ref b)) => {
                let ret = a == b;

                // to detect variants
                if !ret && b.ty == Type::Unknown && b.parent.is_some() && a.parent.is_some() && a.parent == b.parent {
                    if match b.parent {
                        Some(ref p) => p.ty == Type::Struct || p.ty == Type::Enum || p.ty == Type::Use,
                        None => false,
                    } {
                        let mut tmp = b.clone();

                        tmp.ty = Type::Variant;
                        a == &tmp
                    } else {
                        false
                    }
                } else {
                    ret
                }
            },
            _ => false,
        } {
            let mut file_comment = false;

            if elements[pos].1.len() > 0 && elements[pos].1[0].starts_with(FILE_COMMENT) {
                line += 1;
                file_comment = true;
            } else {
                while line > 0 && (line + *decal) > 0 &&
                      original_content[line + *decal - 1].trim().starts_with("#") {
                    line -= 1;
                }
            }
            for comment in &elements[pos].1 {
                if file_comment {
                    original_content.insert(line + *decal, comment[FILE_COMMENT.len()..].to_owned());
                } else {
                    original_content.insert(line + *decal, comment.clone());
                }
                *decal += 1;
            }
            return Some(pos);
        }
        pos += 1;
    }
    None
}

fn regenerate_comments(path: &str, infos: &mut HashMap<String, Vec<(Option<TypeStruct>, Vec<String>)>>) {
    if !infos.contains_key(path) {
        return;
    }
    match strip::build_event_list(path) {
        Ok(mut parse_result) => {
            let mut elements = infos.get_mut(path).unwrap();
            let mut position = 0;
            let mut decal = 0;

            // first, we need to put back file comment
            for entry in elements.iter() {
                if entry.0.is_none() {
                    let mut it = 0;

                    while it < parse_result.original_content.len() {
                        if parse_result.original_content[it].starts_with("/") &&
                           it + 1 < parse_result.original_content.len() &&
                           parse_result.original_content[it + 1].len() < 1 {
                            it += 2;
                            break;
                        }
                        it += 1;
                    }
                    if it < parse_result.original_content.len() {
                        for line in &entry.1 {
                            parse_result.original_content.insert(it, line.clone());
                            decal += 1;
                            it += 1;
                        }
                    }
                    break;
                }
                position += 1;
            }
            if position < elements.len() {
                elements.remove(position);
            }
            let mut waiting_type = None;
            let mut current = None;
            let mut it = 0;

            while it < parse_result.event_list.len() {
                match parse_result.event_list[it].event {
                    EventType::Type(ref t) => {
                        if t.ty != Type::Unknown {
                            waiting_type = Some(t.clone());
                            let tmp = strip::add_to_type_scope(&current, &waiting_type);

                            match get_corresponding_type(&elements, &tmp,
                                                         parse_result.event_list[it].line,
                                                         &mut decal,
                                                         &mut parse_result.original_content) {
                                Some(l) => { elements.remove(l); },
                                None => {}
                            };
                        } else {
                            match current {
                                Some(ref c) => {
                                    if c.ty == Type::Struct || c.ty == Type::Enum ||
                                       c.ty == Type::Mod {
                                        let tmp = Some(t.clone());
                                        let cc = strip::add_to_type_scope(&current, &tmp);

                                        match get_corresponding_type(&elements, &cc,
                                                                     parse_result.event_list[it].line,
                                                                     &mut decal,
                                                                     &mut parse_result.original_content) {
                                            Some(l) => { elements.remove(l); },
                                            None => {}
                                        }
                                    }
                                }
                                None => {}
                            }
                        }
                    }
                    EventType::InScope => {
                        current = strip::add_to_type_scope(&current, &waiting_type);
                        waiting_type = None;
                        match get_corresponding_type(&elements, &current,
                                                     parse_result.event_list[it].line,
                                                     &mut decal,
                                                     &mut parse_result.original_content) {
                            Some(l) => { elements.remove(l); },
                            None => {}
                        };
                    }
                    EventType::OutScope => {
                        current = strip::type_out_scope(&current);
                        waiting_type = None;
                    }
                    _ => {}
                }
                it += 1;
            }
            rewrite_file(path, &parse_result.original_content);
        }
        Err(e) => {
            println!("Error on file '{}': {}", path, e);
        }
    }
}

fn rewrite_file(path: &str, o_content: &[String]) {
    match OpenOptions::new().write(true).create(true).truncate(true).open(path) {
        Ok(mut f) => {
            write!(f, "{}", o_content.join("\n")).unwrap();
        }
        Err(e) => {
            println!("Cannot open '{}': {}", path, e);
        }
    }
}

fn parse_mod_line(line: &str) -> Option<TypeStruct> {
    let line = line.replace(MOD_COMMENT, "");
    let parts : Vec<&str> = line.split("§").collect();
    let mut current = None;

    for part in parts {
        let elems : Vec<&str> = part.split(" ").filter(|x| x.len() > 0).collect();

        current = strip::add_to_type_scope(&current.clone(),
                                           &Some(TypeStruct::new(Type::from(elems[0]),
                                                                 elems[elems.len() - 1])));
    }
    current
}

pub fn regenerate_doc_comments() {
    // we start by storing files info
    let f = match OpenOptions::new().read(true).open("comments.cmts") {
        Ok(f) => f,
        Err(e) => {
            println!("An error occured while trying to open '{}': {}", "comments.cmts", e);
            return;
        }
    };
    let reader = BufReader::new(f);
    let mut current_file = String::new();
    let mut infos = HashMap::new();
    let mut current_infos = vec!();
    let mut lines = vec!();
    let mut it = 0;

    for tmp_line in reader.lines() {
        lines.push(tmp_line.unwrap());
    }
    while it < lines.len() {
        if lines[it].starts_with(FILE) {
            if current_file.len() > 0 && current_infos.len() > 0 {
                infos.insert(current_file, current_infos.clone());
                current_infos = vec!();
            }
            current_file = lines[it].replace(FILE, "").to_owned();
            it += 1;
        } else if lines[it].starts_with(MOD_COMMENT) {
            let ty = parse_mod_line(&lines[it]);
            let mut comments = vec!();

            it += 1;
            if ty.is_none() {
                continue;
            }
            while it < lines.len() {
                if lines[it].starts_with(MOD_COMMENT) ||
                   lines[it].starts_with(FILE) {
                    break;
                }
                comments.push(lines[it].to_owned());
                it += 1;
            }
            if comments.len() > 0 {
                current_infos.push((ty, comments));
            }
        } else if lines[it].starts_with(FILE_COMMENT) {
            let mut comment_lines = vec!();

            while it < lines.len() && lines[it].starts_with(FILE_COMMENT) {
                comment_lines.push(lines[it][FILE_COMMENT.len()..].to_owned());
                it += 1;
            }
            current_infos.push((None, comment_lines));
        }
    }
    if current_file.len() > 0 && current_infos.len() > 0 {
        infos.insert(current_file, current_infos.clone());
    }
    loop_over_files(".", &mut infos, &regenerate_comments);
    // TODO: rewrite comments.cmts with remaining infos in regenerate_comments
}