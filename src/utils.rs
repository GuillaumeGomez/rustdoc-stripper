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

pub fn loop_over_files<T>(path: &str, data: &mut T, func: &Fn(&str, &mut T)) {
    match fs::read_dir(path) {
        Ok(it) => {
            for entry in it {
                check_path_type(entry.unwrap().path().to_str().unwrap(), data, func);
            }
        }
        Err(e) => {
            println!("Error while trying to iterate over {}: {}", path, e);
        }
    }
}

fn check_path_type<T>(path: &str, data: &mut T, func: &Fn(&str, &mut T)) {
    match fs::metadata(path) {
        Ok(m) => {
            if m.is_dir() {
                if path == ".." || path == "." {
                    return;
                }
                loop_over_files(path, data, func);
            } else {
                if path == "./comments.cmts" || !path.ends_with(".rs") {
                    return;
                }
                println!("-> {}", path);
                func(path, data);
            }
        }
        Err(e) => {
            println!("An error occurred on '{}': {}", path, e);
        }
    }
}