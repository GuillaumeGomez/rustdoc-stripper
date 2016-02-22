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

use std::ffi::OsStr;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::path::Path;
use consts::{
    MOD_COMMENT,
    FILE_COMMENT,
    FILE,
    END_INFO,
    OUTPUT_COMMENT_FILE,
};
use types::TypeStruct;

pub fn loop_over_files<S>(path: &Path, func: &mut FnMut(&Path, &str),
    files_to_ignore: &[S], verbose: bool)
where S: AsRef<Path> {
    do_loop_over_files(path.as_ref(), path.as_ref(), func, files_to_ignore, verbose)
}

pub fn do_loop_over_files<S>(work_dir: &Path, path: &Path,
    func: &mut FnMut(&Path, &str), files_to_ignore: &[S], verbose: bool)
where S: AsRef<Path> {
    match fs::read_dir(path) {
        Ok(it) => {
            let mut entries = vec!();

            for entry in it {
                entries.push(entry.unwrap().path().to_owned());
            }
            entries.sort();
            for entry in entries {
                check_path_type(work_dir, &entry, func, files_to_ignore, verbose);
            }
        }
        Err(e) => {
            println!("Error while trying to iterate over {}: {}", path.display(), e);
        }
    }
}

fn check_path_type<S>(work_dir: &Path, path: &Path, func: &mut FnMut(&Path, &str),
    files_to_ignore: &[S], verbose: bool)
where S: AsRef<Path> {
    match fs::metadata(path) {
        Ok(m) => {
            if m.is_dir() {
                if path == Path::new("..") || path == Path::new(".") {
                    return;
                }
                do_loop_over_files(work_dir, path, func, files_to_ignore, verbose);
            } else {
                let path_suffix = strip_prefix(path, work_dir).unwrap();
                let ignore = path == Path::new(&format!("./{}", OUTPUT_COMMENT_FILE)) ||
                    path.extension() != Some(OsStr::new("rs")) ||
                    files_to_ignore.iter().any(|s| s.as_ref() == path_suffix);
                if ignore {
                    if verbose {
                        println!("-> {}: ignored", path.display());
                    }
                    return;
                }
                if verbose {
                    println!("-> {}", path.display());
                }
                func(work_dir, path_suffix.to_str().unwrap());
            }
        }
        Err(e) => {
            println!("An error occurred on '{}': {}", path.display(), e);
        }
    }
}

pub fn join(s: &[String], join_part: &str) -> String {
    let mut ret = String::new();
    let mut it = 0;

    while it < s.len() {
        ret.push_str(&s[it]);
        it += 1;
        if it < s.len() {
            ret.push_str(join_part);
        }
    }
    ret
}

// lifted from libstd for Path::strip_prefix is unstable

fn strip_prefix<'a>(self_: &'a Path, base: &'a Path)
                     -> Result<&'a Path, ()> {
    iter_after(self_.components(), base.components())
        .map(|c| c.as_path())
        .ok_or((()))
}

fn iter_after<A, I, J>(mut iter: I, mut prefix: J) -> Option<I>
    where I: Iterator<Item = A> + Clone,
          J: Iterator<Item = A>,
          A: PartialEq
{
    loop {
        let mut iter_next = iter.clone();
        match (iter_next.next(), prefix.next()) {
            (Some(x), Some(y)) => {
                if x != y {
                    return None;
                }
            }
            (Some(_), None) => return Some(iter),
            (None, None) => return Some(iter),
            (None, Some(_)) => return None,
        }
        iter = iter_next;
    }
}

pub fn write_comment(id: &TypeStruct, comment: &str,
                     ignore_macro: bool) -> String {
    if ignore_macro {
        format!("{}{}{}\n{}", MOD_COMMENT, id, END_INFO, comment)
    } else {
        format!("{}{:?}{}\n{}", MOD_COMMENT, id, END_INFO, comment)
    }
}
pub fn write_item_doc<F>(w: &mut Write, id: &TypeStruct, f: F) -> io::Result<()>
where F: FnOnce(&mut Write) -> io::Result<()> {
    try!(writeln!(w, "{}{}{}", MOD_COMMENT, id, END_INFO));
    f(w)
}

pub fn write_file_comment(comment: &str) -> String {
    format!("{}{}\n{}", FILE_COMMENT, END_INFO, comment)
}

pub fn write_file(file: &str) -> String {
    format!("{}{}{}", FILE, file, END_INFO)
}

pub fn write_file_name(w: &mut Write, name: Option<&str>) -> io::Result<()> {
    writeln!(w, "{}{}{}", FILE, name.unwrap_or("*"), END_INFO)
}
