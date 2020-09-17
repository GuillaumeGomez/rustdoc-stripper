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

use std::fs::File;
use std::io::{self, Read, Write};
use std::ops::Deref;
use std::path::Path;
use std::process::exit;
use types::{EventInfo, EventType, ParseResult, Type, TypeStruct};
use utils::{join, write_comment, write_file, write_file_comment};

const STOP_CHARACTERS: &[char] = &['\t', '\n', '\r', '<', '{', ':', ';', '!', '(', ','];
const COMMENT_ID: &[&str] = &["//", "/*"];
pub(crate) const DOC_COMMENT_ID: &[&str] = &["///", "/*!", "//!", "/**"];
pub(crate) const IGNORE_NEXT_COMMENT: &str = "// rustdoc-stripper-ignore-next";
pub(crate) const IGNORE_NEXT_COMMENT_STOP: &str = "// rustdoc-stripper-ignore-next-stop";

fn move_to(words: &[&str], it: &mut usize, limit: &str, line: &mut usize, start_remove: &str) {
    if words[*it][start_remove.len()..].contains(&limit) {
        return;
    }
    *it += 1;
    while *it < words.len() && !words[*it].contains(limit) {
        if words[*it] == "\n" {
            *line += 1;
        }
        *it += 1;
    }
    if *it < words.len() && words[*it] == "\n" {
        *line += 1;
    }
}

fn move_until(words: &[&str], it: &mut usize, limit: &str, line: &mut usize) {
    let alternative1 = format!("{};", limit);
    let alternative2 = format!("{}\n", limit);
    while *it < words.len()
        && !words[*it].ends_with(limit)
        && !words[*it].ends_with(&alternative1)
        && !words[*it].ends_with(&alternative2)
    {
        *line += words[*it].chars().filter(|c| *c == '\n').count();
        *it += 1;
    }
}

fn get_before<'a>(word: &'a str, limits: &[char]) -> &'a str {
    word.find(limits).map(|pos| &word[..pos]).unwrap_or(word)
}

fn get_impl(words: &[&str], it: &mut usize, line: &mut usize) -> Vec<String> {
    let mut v = vec![];

    while *it + 1 < words.len() {
        if words[*it] == "\n" {
            *line += 1;
        }
        if words[*it + 1] == "{" || words[*it + 1] == ";" {
            break;
        }
        *it += 1;
        v.push(words[*it].to_owned());
    }
    v
}

pub fn add_to_type_scope(
    current: &Option<TypeStruct>,
    e: &Option<TypeStruct>,
) -> Option<TypeStruct> {
    match *current {
        Some(ref c) => match *e {
            Some(ref t) => {
                let mut tmp = t.clone();
                tmp.parent = Some(Box::new(c.clone()));
                Some(tmp)
            }
            _ => {
                let mut tmp = TypeStruct::empty();
                tmp.parent = Some(Box::new(c.clone()));
                Some(tmp)
            }
        },
        None => match *e {
            Some(ref t) => Some(t.clone()),
            _ => None,
        },
    }
}

pub fn type_out_scope(current: &Option<TypeStruct>) -> Option<TypeStruct> {
    match *current {
        Some(ref c) => match c.parent {
            Some(ref p) => Some(p.deref().clone()),
            None => None,
        },
        None => None,
    }
}

fn get_mod(current: &Option<TypeStruct>) -> bool {
    match *current {
        Some(ref t) => {
            if t.ty != Type::Mod {
                println!("Mod/File comments cannot be put here!");
                false
            } else {
                true
            }
        }
        None => true,
    }
}

enum BlockKind<'a> {
    Comment((String, String, &'a str)),
    DocComment((String, String, &'a str)),
    Other(&'a str),
}

fn get_three_parts<'a>(
    before: &'a str,
    comment_sign: &str,
    after: &'a str,
    stop: &str,
) -> (String, String, &'a str) {
    if let Some(pos) = after.find(stop) {
        let extra = if stop != "\n" { stop.len() } else { 0 };
        (
            before.to_owned(),
            format!("{} {}", comment_sign, &after[0..pos]),
            &after[pos + extra..],
        )
    } else {
        (
            before.to_owned(),
            format!("{} {}", comment_sign, &after),
            &after[after.len() - 1..],
        )
    }
}

fn check_if_should_be_ignored(text: &str) -> bool {
    let mut ignore_until_multi_end = false;
    for line in text.split('\n').rev() {
        let line = line.trim();
        if !ignore_until_multi_end && !line.starts_with("//") {
            if line.trim().ends_with("*/") {
                ignore_until_multi_end = !line.starts_with("/**") && !line.starts_with("/*!");
                continue;
            }
        } else if line.starts_with("/*") {
            ignore_until_multi_end = false;
            continue;
        }
        if !ignore_until_multi_end {
            if line == IGNORE_NEXT_COMMENT_STOP {
                return false;
            } else if line == IGNORE_NEXT_COMMENT {
                return true;
            }
            if !line.starts_with("///") && !line.starts_with("//!") {
                break;
            }
        }
    }
    false
}

fn find_one_of<'a>(comments: &[&str], doc_comments: &[&str], text: &'a str) -> BlockKind<'a> {
    let mut last_pos = 0;

    let mut tmp_text = &text[last_pos..];
    while let Some(pos) = tmp_text.find('/') {
        tmp_text = &tmp_text[pos..];
        last_pos += pos;
        for com in doc_comments {
            if tmp_text.starts_with(com) {
                if &com[1..2] == "*" {
                    return BlockKind::DocComment(get_three_parts(
                        &text[0..last_pos],
                        com,
                        &text[last_pos + com.len()..],
                        "*/",
                    ));
                } else {
                    return BlockKind::DocComment(get_three_parts(
                        &text[0..last_pos],
                        com,
                        &text[last_pos + com.len()..],
                        "\n",
                    ));
                }
            }
        }
        for com in comments {
            if tmp_text.starts_with(com) {
                if &com[1..2] == "*" {
                    return BlockKind::Comment(get_three_parts(
                        &text[0..last_pos],
                        "",
                        &text[last_pos..],
                        "*/",
                    ));
                } else {
                    return BlockKind::Comment(get_three_parts(
                        &text[0..last_pos],
                        "",
                        &text[last_pos..],
                        "\n",
                    ));
                }
            }
        }
        if pos + 1 < tmp_text.len() {
            tmp_text = &text[pos + 1..];
        } else {
            break;
        }
    }
    BlockKind::Other(text)
}

fn transform_code(code: &str) -> String {
    code.replace("{", " { ")
        .replace("}", " } ")
        .replace(":", " : ")
        .replace(" :  : ", "::")
        .replace("*/", " */")
        .replace("\n", " \n ")
        .replace("!(", " !! (")
        .replace("!  {", " !? {")
        .replace(",", ", ")
        .replace("(", " (")
}

// Replaces lines that should be removed (doc comments mostly) with empty lines to keep a working
// line match.
fn clean_input(s: &str) -> String {
    let mut ret = String::new();
    let mut text = s;
    loop {
        text = match find_one_of(COMMENT_ID, DOC_COMMENT_ID, text) {
            BlockKind::Other(content) => {
                ret.push_str(&transform_code(content));
                break;
            }
            BlockKind::DocComment((before, doc_comment, after))
                if !check_if_should_be_ignored(&s[..s.len() - after.len()]) =>
            {
                ret.push_str(&transform_code(&before));
                ret.push_str(&doc_comment);
                after
            }
            BlockKind::DocComment((before, doc_comment, after)) => {
                ret.push_str(&transform_code(&before));
                for _ in 0..doc_comment.split('\n').count() - 1 {
                    ret.push_str(" \n ");
                }
                after
            }
            BlockKind::Comment((before, comment, after)) => {
                ret.push_str(&transform_code(&before));
                for _ in 0..comment.split('\n').count() - 1 {
                    ret.push_str(" \n ");
                }
                after
            }
        };
    }
    ret
}

fn clear_events(mut events: Vec<EventInfo>) -> Vec<EventInfo> {
    let mut current: Option<TypeStruct> = None;
    let mut waiting_type: Option<TypeStruct> = None;
    let mut it = 0;

    while it < events.len() {
        if match events[it].event {
            EventType::Type(ref t) => {
                if t.ty != Type::Unknown {
                    waiting_type = Some(t.clone());
                    false
                } else if let Some(ref parent) = current {
                    match parent.ty {
                        Type::Struct | Type::Enum => false,
                        _ => true,
                    }
                } else {
                    true
                }
            }
            EventType::InScope => {
                current = add_to_type_scope(&current, &waiting_type);
                waiting_type = None;
                false
            }
            EventType::OutScope => {
                current = type_out_scope(&current);
                waiting_type = None;
                false
            }
            _ => false,
        } {
            events.remove(it);
            continue;
        }
        it += 1;
    }
    events
}

fn remove_stop_chars(s: &str) -> String {
    let mut s = s.to_owned();
    for c in STOP_CHARACTERS {
        if s.contains(*c) {
            s = s.replace(&c.to_string(), "");
        }
    }
    s
}

#[allow(clippy::useless_let_if_seq)]
fn build_event_inner(
    it: &mut usize,
    line: &mut usize,
    words: &[&str],
    event_list: &mut Vec<EventInfo>,
    comment_lines: &mut Vec<usize>,
    b_content: &[String],
    mut par_count: Option<isize>,
) {
    let mut waiting_for_macro = false;
    while *it < words.len() {
        match words[*it] {
            c if c.starts_with('\"') => move_to(&words, it, "\"", line, "\""),
            c if c.starts_with("b\"") => move_to(&words, it, "\"", line, "b\""),
            // c if c.starts_with("'") => move_to(&words, it, "'", line),
            c if c.starts_with("r#") => {
                let end = c
                    .split("#\"")
                    .next()
                    .unwrap()
                    .replace("\"", "")
                    .replace("r", "");
                move_to(&words, it, &format!("\"{}", end), line, "r#");
            }
            "///" | "///\n" => {
                comment_lines.push(*line);
                event_list.push(EventInfo::new(
                    *line,
                    EventType::Comment(b_content[*line].to_owned()),
                ));
                move_to(&words, it, "\n", line, "");
            }
            "//!" | "//!\n" => {
                comment_lines.push(*line);
                event_list.push(EventInfo::new(
                    *line,
                    EventType::FileComment(b_content[*line].to_owned()),
                ));
                if *line + 1 < b_content.len() && b_content[*line + 1].is_empty() {
                    comment_lines.push(*line + 1);
                }
                move_to(&words, it, "\n", line, "");
            }
            "/*!" | "/*!\n" => {
                let mark = *line;
                move_until(&words, it, "*/", line);
                for (pos, s) in b_content.iter().enumerate().take(*line).skip(mark) {
                    comment_lines.push(pos);
                    event_list.push(EventInfo::new(*line, EventType::FileComment(s.to_owned())));
                }
                comment_lines.push(*line);
                let mut removed = false;
                if *line + 1 < b_content.len() && b_content[*line + 1].is_empty() {
                    comment_lines.push(*line + 1);
                    removed = true;
                }
                event_list.push(EventInfo::new(
                    mark,
                    EventType::FileComment("*/".to_owned()),
                ));
                if removed {
                    event_list.push(EventInfo::new(*line, EventType::FileComment("".to_owned())));
                }
            }
            "/**" | "/**\n" => {
                let mark = *line;
                move_until(&words, it, "*/", line);
                for (pos, s) in b_content.iter().enumerate().take(*line).skip(mark) {
                    comment_lines.push(pos);
                    event_list.push(EventInfo::new(*line, EventType::Comment(s.to_owned())));
                }
                comment_lines.push(*line);
                let mut removed = false;
                if *line + 1 < b_content.len() && b_content[*line + 1].is_empty() {
                    comment_lines.push(*line + 1);
                    removed = true;
                }
                event_list.push(EventInfo::new(
                    mark,
                    EventType::FileComment("*/".to_owned()),
                ));
                if removed {
                    event_list.push(EventInfo::new(*line, EventType::Comment("".to_owned())));
                }
            }
            "use" | "mod" => {
                let mut name = words[*it + 1].to_owned();
                let ty = words[*it];

                if *line + 1 < b_content.len() && b_content[*line].ends_with("::{") {
                    move_to(&words, it, "\n", line, "");
                    name.push_str(&b_content[*line + 1].trim());
                }
                event_list.push(EventInfo::new(
                    *line,
                    EventType::Type(TypeStruct::new(Type::from(ty), &name)),
                ));
            }
            "struct" | "fn" | "enum" | "const" | "static" | "type" | "trait" | "macro_rules!"
            | "flags" => {
                if *it + 1 >= words.len() {
                    break;
                }
                event_list.push(EventInfo::new(
                    *line,
                    EventType::Type(TypeStruct::new(
                        Type::from(words[*it]),
                        get_before(words[*it + 1], STOP_CHARACTERS),
                    )),
                ));
                waiting_for_macro = words[*it] == "macro_rules!";
                *it += 1;
            }
            "!!" => {
                event_list.push(EventInfo::new(
                    *line,
                    EventType::Type(TypeStruct::new(
                        Type::from("macro"),
                        &format!("{}!{}", words[*it - 1], words[*it + 1]),
                    )),
                ));
                *it += 1;
            }
            "!?" => {
                event_list.push(EventInfo::new(
                    *line,
                    EventType::Type(TypeStruct::new(
                        Type::from("macro"),
                        &format!("{}!", words[*it - 1]),
                    )),
                ));
            }
            "impl" => {
                event_list.push(EventInfo::new(
                    *line,
                    EventType::Type(TypeStruct::new(
                        Type::Impl,
                        &join(&get_impl(&words, it, line), " "),
                    )),
                ));
            }
            c if c.starts_with("impl<") => {
                event_list.push(EventInfo::new(
                    *line,
                    EventType::Type(TypeStruct::new(
                        Type::Impl,
                        &join(&get_impl(&words, it, line), " "),
                    )),
                ));
            }
            x if x == "{" || x == "{\n" => {
                if let Some(ref mut par_count) = par_count {
                    *par_count += 1;
                }
                event_list.push(EventInfo::new(*line, EventType::InScope));
                if waiting_for_macro {
                    build_event_inner(
                        it,
                        line,
                        &words,
                        &mut vec![],
                        &mut vec![],
                        &b_content,
                        Some(1),
                    );
                    waiting_for_macro = false;
                }
                if x == "}\n" {
                    *line += 1;
                }
            }
            x if x == "}" || x == "}\n" => {
                if let Some(ref mut par_count) = par_count {
                    *par_count -= 1;
                    if *par_count <= 0 {
                        return;
                    }
                }
                event_list.push(EventInfo::new(*line, EventType::OutScope));
                if x == "}\n" {
                    *line += 1;
                }
            }
            "\n" => {
                *line += 1;
            }
            s if s.starts_with("#[") || s.starts_with("#![") => {
                while *it < words.len() {
                    *line += words[*it].split('\n').count() - 1;
                    if words[*it].contains(']') {
                        break;
                    }
                    *it += 1;
                }
                *line += s.chars().filter(|c| *c == '\n').count();
            }
            x => {
                event_list.push(EventInfo::new(
                    *line,
                    EventType::Type(TypeStruct::new(
                        Type::Unknown,
                        &remove_stop_chars(words[*it]),
                    )),
                ));
                *line += x.chars().filter(|c| *c == '\n').count();
            }
        }
        *it += 1;
    }
}

pub fn build_event_list(path: &Path) -> io::Result<ParseResult> {
    let mut f = File::open(path)?;
    let mut b_content = String::new();
    f.read_to_string(&mut b_content).unwrap();
    let content = clean_input(&b_content);
    let b_content: Vec<String> = b_content.split('\n').map(|s| s.to_owned()).collect();
    let words: Vec<&str> = content.split(' ').filter(|s| !s.is_empty()).collect();
    let mut it = 0;
    let mut line = 0;
    let mut event_list = vec![];
    let mut comment_lines = vec![];

    build_event_inner(
        &mut it,
        &mut line,
        &words,
        &mut event_list,
        &mut comment_lines,
        &b_content,
        None,
    );
    let clear = clear_events(event_list);
    Ok(ParseResult {
        event_list: clear,
        comment_lines,
        original_content: b_content,
    })
}

fn unformat_comment(c: &str) -> String {
    fn remove_prepend(s: &str) -> String {
        let mut s = s.to_owned();

        for to_remove in DOC_COMMENT_ID {
            s = s.replace(to_remove, "");
        }
        /*for to_remove in COMMENT_ID {
            s = s.replace(to_remove, "");
        }*/
        if s.starts_with(' ') {
            (&s)[1..].to_owned()
        } else {
            s
        }
    }

    c.replace("*/", "")
        .split('\n')
        .map(|s| remove_prepend(s.trim_start()))
        .collect::<Vec<String>>()
        .join("\n")
}

pub fn strip_comments<F: Write>(
    work_dir: &Path,
    path: &str,
    out_file: &mut F,
    ignore_macros: bool,
) {
    let full_path = work_dir.join(path);
    match build_event_list(&full_path) {
        Ok(parse_result) => {
            if parse_result.comment_lines.is_empty() {
                return;
            }
            writeln!(out_file, "{}", &write_file(path)).unwrap();
            let mut current: Option<TypeStruct> = None;
            let mut waiting_type: Option<TypeStruct> = None;
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
                        if !get_mod(&current) {
                            exit(1);
                        }
                        it += 1;
                        let mut comments = format!(
                            "{}\n",
                            &write_file_comment(&unformat_comment(c), &current, ignore_macros)
                        );
                        while parse_result
                            .event_list
                            .get(it)
                            .map(|x| match x.event {
                                EventType::FileComment(ref c) => {
                                    comments.push_str(&format!("{}\n", unformat_comment(c)));
                                    true
                                }
                                _ => false,
                            })
                            .unwrap_or(false)
                        {
                            it += 1;
                        }
                        write!(out_file, "{}", comments).unwrap();
                        continue;
                    }
                    EventType::Comment(ref c) => {
                        let mut comments = format!("{}\n", c);

                        it += 1;
                        while it < parse_result.event_list.len()
                            && match parse_result.event_list[it].event {
                                EventType::Comment(ref c) => {
                                    comments.push_str(&format!("{}\n", c));
                                    true
                                }
                                EventType::Type(_) => false,
                                _ => panic!("Doc comments cannot be written everywhere"),
                            }
                        {
                            it += 1;
                        }
                        while parse_result
                            .event_list
                            .get(it)
                            .map(|x| match x.event {
                                EventType::Type(ref t) => match t.ty {
                                    Type::Unknown => match current {
                                        Some(ref cur) => {
                                            if cur.ty == Type::Enum
                                                || cur.ty == Type::Struct
                                                || cur.ty == Type::Use
                                            {
                                                if t.name == "pub" {
                                                    true
                                                } else {
                                                    let mut copy = t.clone();
                                                    copy.ty = Type::Variant;
                                                    let tmp =
                                                        add_to_type_scope(&current, &Some(copy));
                                                    write!(
                                                        out_file,
                                                        "{}",
                                                        write_comment(
                                                            &tmp.unwrap(),
                                                            &unformat_comment(&comments),
                                                            ignore_macros
                                                        )
                                                    )
                                                    .unwrap();
                                                    false
                                                }
                                            } else {
                                                t.name == "pub"
                                            }
                                        }
                                        None => t.name == "pub",
                                    },
                                    _ => {
                                        let tmp = add_to_type_scope(&current, &Some(t.clone()));
                                        write!(
                                            out_file,
                                            "{}",
                                            write_comment(
                                                &tmp.unwrap(),
                                                &unformat_comment(&comments),
                                                ignore_macros
                                            )
                                        )
                                        .unwrap();
                                        false
                                    }
                                },
                                _ => panic!("An item was expected for this comment: {}", comments),
                            })
                            .unwrap_or(false)
                        {
                            it += 1;
                        }
                        continue;
                    }
                }
                it += 1;
            }
            // we now remove doc comments from original file
            remove_comments(
                &full_path,
                &parse_result.comment_lines,
                parse_result.original_content,
            );
        }
        Err(e) => {
            println!("Unable to open \"{}\": {}", path, e);
        }
    }
}

fn remove_comments(path: &Path, to_remove: &[usize], mut o_content: Vec<String>) {
    let mut decal = 0;
    match File::create(path) {
        Ok(mut f) => {
            for line in to_remove.iter() {
                if line - decal > 0
                    && line - decal + 1 < o_content.len()
                    && o_content[line - decal - 1].trim() == IGNORE_NEXT_COMMENT_STOP
                {
                    let l = o_content[line - decal + 1].trim();
                    if DOC_COMMENT_ID.iter().any(|d| l.starts_with(d)) {
                        o_content.remove(line - decal - 1);
                        decal += 1;
                    }
                }
                o_content.remove(line - decal);
                decal += 1;
            }
            write!(f, "{}", o_content.join("\n")).unwrap();
        }
        Err(e) => {
            println!("Cannot open '{}': {}", path.display(), e);
        }
    }
}
