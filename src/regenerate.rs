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

use std::fs::{File, OpenOptions, remove_file};
use std::io::{BufRead, BufReader, Write};
use std::collections::HashMap;
use std::ops::Deref;
use std::path::Path;
use strip;
use types::{EventType, ParseResult, Type, TypeStruct};
use utils::{join, loop_over_files, write_comment, write_file};
use std::iter;
use consts::{
    MOD_COMMENT,
    FILE_COMMENT,
    FILE,
    END_INFO,
};

type Infos = HashMap<Option<String>, Vec<(Option<TypeStruct>, Vec<String>)>>;

fn gen_indent(indent: usize) -> String {
    iter::repeat("    ").take(indent).collect::<Vec<&str>>().join("")
}

fn gen_indent_from(from: &str) -> String {
    for (i, c) in from.chars().enumerate() {
        if c != ' ' && c != '\t' {
            return gen_indent(i / 4);
        }
    }
    String::new()
}

fn regenerate_comment(is_file_comment: bool, position: usize, indent: usize, comment: &str,
                      original_content: &mut Vec<String>) {
    let is_empty = comment.trim().is_empty();
    let read_indent = if is_file_comment {
        gen_indent(indent)
    } else {
        let tmp = original_content[position].clone();
        gen_indent_from(&tmp)
    };
    original_content.insert(position,
                            format!("{}{}{}{}",
                                    &read_indent,
                                    if is_file_comment { "//!" } else { "///" },
                                    if is_empty { "" } else { " " },
                                    if is_empty { "" } else { &comment }));
}

#[allow(clippy::useless_let_if_seq)]
fn get_corresponding_type(elements: &[(Option<TypeStruct>, Vec<String>)],
                          to_find: &Option<TypeStruct>,
                          mut line: usize,
                          decal: &mut usize,
                          original_content: &mut Vec<String>,
                          ignore_macros: bool) -> Option<usize> {
    let mut pos = 0;

    while pos < elements.len() {
        if match (&elements[pos].0, to_find) {
            (&Some(ref a), &Some(ref b)) => {
                let ret = a == b;

                // to detect variants
                if !ret && b.ty == Type::Unknown && b.parent.is_some() && a.parent.is_some() &&
                   a.parent == b.parent {
                    if match b.parent {
                        Some(ref p) => {
                            p.ty == Type::Struct || p.ty == Type::Enum || p.ty == Type::Use
                        }
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

            if !elements[pos].1.is_empty() && elements[pos].1[0].starts_with("//!") {
                line += 1;
                file_comment = true;
            } else {
                while line > 0 && (line + *decal) > 0 &&
                      original_content[line + *decal - 1].trim_start().starts_with('#') {
                    line -= 1;
                }
            }
            for comment in (&elements[pos].1).iter().skip(if file_comment { 1 } else { 0 }) {
                let depth = if let Some(ref e) = elements[pos].0 {
                    e.get_depth(ignore_macros)
                } else {
                    0
                };
                regenerate_comment(file_comment, line + *decal, depth + 1, &comment,
                                   original_content);
                *decal += 1;
            }
            return Some(pos);
        }
        pos += 1;
    }
    None
}

// The hashmap key is `Some(file name)` or `None` for entries that ignore file name
pub fn regenerate_comments(work_dir: &Path, path: &str,
        infos: &mut Infos,
        ignore_macros: bool, ignore_doc_commented: bool) {
    if !infos.contains_key(&None) && !infos.contains_key(&Some(path.to_owned())) {
        return;
    }
    let full_path = work_dir.join(path);
    match strip::build_event_list(&full_path) {
        Ok(ref mut parse_result) => {
            // exact path match
            if let Some(v) = infos.get_mut(&Some(path.to_owned())) {
                do_regenerate(&full_path, parse_result, v, ignore_macros, ignore_doc_commented);
            }
            // apply to all files
            if let Some(v) = infos.get_mut(&None) {
                do_regenerate(&full_path, parse_result, v, ignore_macros, ignore_doc_commented);
            }
        }
        Err(e) => {
            println!("Error in file '{}': {}", path, e);
        }
    }
}

fn check_if_regen(it: usize, parse_result: &ParseResult, ignore_doc_commented: bool) -> bool {
    ignore_doc_commented && it > 0 &&
    match parse_result.event_list[it - 1].event {
        EventType::Comment(_) | EventType::FileComment(_) => true,
        _ => false,
    }
}

fn do_regenerate(path: &Path, parse_result: &mut ParseResult,
                 elements: &mut Vec<(Option<TypeStruct>, Vec<String>)>,
                 ignore_macros: bool, ignore_doc_commented: bool) {
    let mut position = 0;
    let mut decal = 0;

    // first, we need to put back file comment
    for entry in elements.iter() {
        if entry.0.is_none() {
            let mut it = 0;

            while it < parse_result.original_content.len() &&
                  parse_result.original_content[it].starts_with('/') {
                it += 1;
            }
            if it > 0 {
                it += 1;
            }
            if it < parse_result.original_content.len() {
                for line in &entry.1 {
                    if line.trim().is_empty() {
                        parse_result.original_content.insert(it, "//!".to_string());
                    } else {
                        parse_result.original_content.insert(it, format!("//! {}", &line));
                    }
                    decal += 1;
                    it += 1;
                }
            }
            parse_result.original_content.insert(it, "".to_owned());
            decal += 1;
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
                    let tmp = {
                        let t = strip::add_to_type_scope(&current, &waiting_type);
                        if ignore_macros {
                            erase_macro_path(t)
                        } else {
                            t
                        }
                    };

                    if !check_if_regen(it, parse_result, ignore_doc_commented) {
                        if let Some(l) = get_corresponding_type(&elements, &tmp,
                                                     parse_result.event_list[it].line,
                                                     &mut decal,
                                                     &mut parse_result.original_content,
                                                     ignore_macros) {
                            elements.remove(l);
                        }
                    }
                } else if let Some(ref c) = current {
                    if c.ty == Type::Struct ||
                       c.ty == Type::Enum ||
                       c.ty == Type::Mod {
                        let tmp = Some(t.clone());
                        let cc = {
                            let t = strip::add_to_type_scope(&current, &tmp);
                            if ignore_macros {
                                erase_macro_path(t)
                            } else {
                                t
                            }
                        };

                        if !check_if_regen(it, parse_result, ignore_doc_commented) {
                            if let Some(l) = get_corresponding_type(&elements, &cc,
                                                         parse_result.event_list[it].line,
                                                         &mut decal,
                                                         &mut parse_result.original_content,
                                                         ignore_macros) {
                                elements.remove(l);
                            }
                        }
                    }
                }
            }
            EventType::InScope => {
                current = strip::add_to_type_scope(&current, &waiting_type);
                waiting_type = None;
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

fn rewrite_file(path: &Path, o_content: &[String]) {
    match File::create(path) {
        Ok(mut f) => {
            write!(f, "{}", o_content.join("\n")).unwrap();
        }
        Err(e) => {
            println!("Cannot open '{}': {}", path.display(), e);
        }
    }
}

fn parse_mod_line(line: &str) -> Option<TypeStruct> {
    let line = line.replace(FILE_COMMENT, "").replace(MOD_COMMENT, "").replace(END_INFO, "");
    if line.is_empty() {
        return None
    }
    let parts : Vec<&str> = line.split("::").collect();
    let mut current = None;

    for part in parts {
        let elems : Vec<&str> = part.split(' ').filter(|x| !x.is_empty()).collect();

        current = strip::add_to_type_scope(&current.clone(),
                                           &Some(TypeStruct::new(Type::from(elems[0]),
                                                                 elems[elems.len() - 1])));
    }
    current
}

fn save_remainings(infos: &Infos, comment_file: &str) {
    let mut remainings = 0;

    for content in infos.values() {
        if !content.is_empty() {
            remainings += 1;
        }
    }
    if remainings < 1 {
        let _ = remove_file(comment_file);
        return;
    }
    match File::create(comment_file) {
        Ok(mut out_file) => {
            println!("Some comments haven't been regenerated to the files. Saving them \
                      back to '{}'.",
                     comment_file);
            for (key, content) in infos {
                if content.is_empty() {
                    continue;
                }
                // Set the name to "*" for entries that ignore file name
                let key = key.as_ref().map(|s| &s[..]).unwrap_or("*");
                let _ = writeln!(out_file, "{}", &write_file(key));
                for line in content {
                    if let Some(ref d) = line.0 {
                        let _ = writeln!(out_file,
                                         "{}", write_comment(d, &join(&line.1, "\n"),
                                         false));
                    }
                }
            }
        },
        Err(e) => {
            println!("An error occured while trying to open '{}': {}", comment_file, e);
        },
    }
}

pub fn regenerate_doc_comments(directory: &str, verbose: bool, comment_file: &str,
                               ignore_macros: bool,
                               ignore_doc_commented: bool) {
    // we start by storing files info
    let f = match OpenOptions::new().read(true).open(comment_file) {
        Ok(f) => f,
        Err(e) => {
            println!("An error occured while trying to open '{}': {}", comment_file, e);
            return;
        },
    };
    let reader = BufReader::new(f);
    let lines = reader.lines().map(|line| line.unwrap());
    let mut infos = parse_cmts(lines, ignore_macros);
    let ignores: &[&str] = &[];

    loop_over_files(directory.as_ref(), &mut |w, s| {
        regenerate_comments(w, s, &mut infos, ignore_macros, ignore_doc_commented)
    }, &ignores, verbose);
    save_remainings(&infos, comment_file);
}

fn sub_erase_macro_path(ty: Option<Box<TypeStruct>>, is_parent: bool) -> Option<Box<TypeStruct>> {
    match ty {
        Some(ref t) if is_parent => {
            if t.ty == Type::Macro {
                sub_erase_macro_path(t.clone().parent, true)
            } else {
                let mut tmp = t.clone();
                tmp.parent = sub_erase_macro_path(t.clone().parent, true);
                Some(tmp)
            }
        }
        Some(t) => {
            let mut tmp = t.clone();
            tmp.parent = sub_erase_macro_path(t.parent, true);
            Some(tmp)
        }
        None => None,
    }
}

fn erase_macro_path(ty: Option<TypeStruct>) -> Option<TypeStruct> {
    if let Some(t) = ty {
        Some(*sub_erase_macro_path(Some(Box::new(t)), false).unwrap())
    } else {
        None
    }
}

pub fn parse_cmts<S, I>(lines: I, ignore_macros: bool) -> Infos
where S: Deref<Target = str>,
      I: Iterator<Item = S> {
    enum State {
        Initial,
        File {
            file: Option<String>,
            infos: Vec<(Option<TypeStruct>, Vec<String>)>,
            ty: Option<TypeStruct>,
            comments: Vec<String>,
        }
    }

    // Returns `Some(name)` if the line matches FILE
    // where name is Some for an actual file name and None for "*"
    // The "*" entries are to be applied regardless of file name
    #[allow(clippy::option_option)]
    fn line_file(line: &str) -> Option<Option<String>> {
        if line.starts_with(FILE) {
            let name = &line[FILE.len()..].replace(END_INFO, "");
            if name == "*" {
                Some(None)
            }
            else {
                Some(Some(name.to_owned()))
            }
        } else {
            None
        }
    }

    let mut ret = HashMap::new();
    let mut state = State::Initial;

    for line in lines {
        state = match state {
            State::Initial => {
                if let Some(file) = line_file(&line) {
                    State::File {
                        file,
                        infos: vec![],
                        ty: None,
                        comments: vec![],
                    }
                } else {
                    panic!("Unrecognized format");
                }
            },
            State::File { mut file, mut infos, mut ty, mut comments } => {
                if let Some(new_file) = line_file(&line) {
                    if !comments.is_empty() {
                        infos.push((ty.take(), comments));
                        comments = vec![];
                    }
                    if !infos.is_empty() {
                        ret.insert(file, infos);
                        file = new_file;
                        infos = vec![];
                    }
                } else if line.starts_with(FILE_COMMENT) {
                    if let Some(ty) = ty.take() {
                        if !comments.is_empty() {
                            infos.push((Some(ty), comments));
                            comments = vec!["//!".to_owned()];
                        }
                    } else if !comments.is_empty() {
                        infos.push((None, comments));
                        comments = vec![];
                    }
                    ty = parse_mod_line(&line[..]);
                } else if line.starts_with(MOD_COMMENT) {
                    if !comments.is_empty() {
                        infos.push((ty, comments));
                        comments = vec![];
                    }
                    ty = parse_mod_line(&line[..]);
                } else {
                    comments.push(line[..].to_owned());
                }
                State::File {
                    file,
                    infos,
                    ty: if ignore_macros { erase_macro_path(ty) } else { ty },
                    comments,
                }
            },
        }
    }

    if let State::File { file, mut infos, ty, comments } = state {
        if !comments.is_empty() {
            infos.push((ty, comments));
        }
        if !infos.is_empty() {
            ret.insert(file, infos);
        }
    }

    ret
}
