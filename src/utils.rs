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

use std::fs;
use std::ops::Deref;

pub fn loop_over_files<T, S>(path: &str, data: &mut T, func: &Fn(&str, &mut T),
    files_to_ignore: &[S], verbose: bool)
where S: Deref<Target = str> {
    match fs::read_dir(path) {
        Ok(it) => {
            let mut entries = vec!();

            for entry in it {
                let path = entry.unwrap().path().to_str().unwrap().to_owned();

                entries.push(path.clone());
            }
            entries.sort();
            for entry in entries {
                check_path_type(&entry, data, func,
                    files_to_ignore, verbose);
            }
        }
        Err(e) => {
            println!("Error while trying to iterate over {}: {}", path, e);
        }
    }
}

fn check_path_type<T, S>(path: &str, data: &mut T, func: &Fn(&str, &mut T),
    files_to_ignore: &[S], verbose: bool)
where S: Deref<Target = str> {
    match fs::metadata(path) {
        Ok(m) => {
            if m.is_dir() {
                if path == ".." || path == "." {
                    return;
                }
                loop_over_files(path, data, func, files_to_ignore, verbose);
            } else {
                if path == "./comments.cmts" || !path.ends_with(".rs") ||
                   files_to_ignore.iter().any(|s| &s[..] == path) {
                    if verbose {
                        println!("-> {}: ignored", path);
                    }
                    return;
                }
                if verbose {
                    println!("-> {}", path);
                }
                func(path, data);
            }
        }
        Err(e) => {
            println!("An error occurred on '{}': {}", path, e);
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
