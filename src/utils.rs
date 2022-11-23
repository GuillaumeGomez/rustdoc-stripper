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

use consts::{END_INFO, FILE, FILE_COMMENT, MOD_COMMENT, OUTPUT_COMMENT_FILE};
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::path::Path;
use types::TypeStruct;

use crate::Type;

pub fn loop_over_files<S>(
    path: &Path,
    func: &mut dyn FnMut(&Path, &str),
    files_to_ignore: &[S],
    verbose: bool,
) where
    S: AsRef<Path>,
{
    do_loop_over_files(path, path, func, files_to_ignore, verbose)
}

pub fn do_loop_over_files<S>(
    work_dir: &Path,
    path: &Path,
    func: &mut dyn FnMut(&Path, &str),
    files_to_ignore: &[S],
    verbose: bool,
) where
    S: AsRef<Path>,
{
    match fs::read_dir(path) {
        Ok(it) => {
            let mut entries = vec![];

            for entry in it {
                entries.push(entry.unwrap().path().to_owned());
            }
            entries.sort();
            for entry in entries {
                check_path_type(work_dir, &entry, func, files_to_ignore, verbose);
            }
        }
        Err(e) => {
            println!(
                "Error while trying to iterate over {}: {}",
                path.display(),
                e
            );
        }
    }
}

fn check_path_type<S>(
    work_dir: &Path,
    path: &Path,
    func: &mut dyn FnMut(&Path, &str),
    files_to_ignore: &[S],
    verbose: bool,
) where
    S: AsRef<Path>,
{
    match fs::metadata(path) {
        Ok(m) => {
            if m.is_dir() {
                if path == Path::new("..") || path == Path::new(".") {
                    return;
                }
                do_loop_over_files(work_dir, path, func, files_to_ignore, verbose);
            } else {
                let path_suffix = strip_prefix(path, work_dir).unwrap();
                let ignore = path == Path::new(&format!("./{}", OUTPUT_COMMENT_FILE))
                    || path.extension() != Some(OsStr::new("rs"))
                    || files_to_ignore.iter().any(|s| s.as_ref() == path_suffix);
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

fn strip_prefix<'a>(self_: &'a Path, base: &'a Path) -> Result<&'a Path, ()> {
    iter_after(self_.components(), base.components())
        .map(|c| c.as_path())
        .ok_or(())
}

fn iter_after<A, I, J>(mut iter: I, mut prefix: J) -> Option<I>
where
    I: Iterator<Item = A> + Clone,
    J: Iterator<Item = A>,
    A: PartialEq,
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

pub fn write_comment(id: &TypeStruct, comment: &str, ignore_macro: bool) -> String {
    if ignore_macro {
        format!("{}{}{}\n{}", MOD_COMMENT, id, END_INFO, comment)
    } else {
        format!("{}{:?}{}\n{}", MOD_COMMENT, id, END_INFO, comment)
    }
}
pub fn write_item_doc<F>(w: &mut dyn Write, id: &TypeStruct, f: F) -> io::Result<()>
where
    F: FnOnce(&mut dyn Write) -> io::Result<()>,
{
    writeln!(w, "{}{}{}", MOD_COMMENT, id, END_INFO)?;
    f(w)
}

pub fn write_file_comment(comment: &str, id: &Option<TypeStruct>, ignore_macro: bool) -> String {
    if let Some(ref t) = *id {
        if ignore_macro {
            format!("{} {}{}\n{}", FILE_COMMENT, t, END_INFO, comment)
        } else {
            format!("{} {:?}{}\n{}", FILE_COMMENT, t, END_INFO, comment)
        }
    } else {
        format!("{}{}\n{}", FILE_COMMENT, END_INFO, comment)
    }
}

pub fn write_file(file: &str) -> String {
    format!("{}{}{}", FILE, file, END_INFO)
}

pub fn write_file_name(w: &mut dyn Write, name: Option<&str>) -> io::Result<()> {
    writeln!(w, "{}{}{}", FILE, name.unwrap_or("*"), END_INFO)
}

/// If the [`TypeStruct`]'s oldest parent (the parent with a *parent*=None) is a [`Type::Macro`][crate::Type::Macro],
/// this funciton will remove that parent.
///
/// Useful when trying to compare equality of a `struct SomeStruct` in the doc file,
/// and `SomeStruct` in the source code is wrapped inside a macro.
/// In the resulting code, `SomeStruct` would end up outside of the macro,
/// so even if the perceived path of `SomeStruct` is `macro SomeMacro::struct SomeStruct`,
/// the content from `struct SomeStruct` has to be written there.
///
/// This funciton is recursive.
pub fn remove_macro_parent(tys: &mut TypeStruct) {
    if let Some(parent) = &mut tys.parent {
        if parent.ty == Type::Macro {
            tys.parent = None;
        } else {
            remove_macro_parent(parent)
        }
    }
}
