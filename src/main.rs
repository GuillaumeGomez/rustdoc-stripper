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

use std::{env, fs};
use std::fs::File;

fn loop_over_files(path: &str, saver: &File) {
    match fs::read_dir(path) {
        Ok(it) => {
            for entry in it {
                check_path_type(entry.path().to_str().unwrap(), saver);
            }
        }
        Err(e) => {
            println!("Error while trying to iterate over {}: {}", path, e);
        }
    }
}

fn strip_comments(path: &str, saver: &File) {
    match File::open(path) {
        Ok(f) => {
            let mut reader = BufReader::new(f);
            let mut line = String::new();

        }
        Err(e) => {
            println!("Unable to open \"{}\": {}", path, e);
        }
    }
}

fn check_path_type(path: &str, saver: &File) {
    match fs::metadata(path) {
        Ok(m) => {
            if m.is_dir() {
                loop_over_files(path, saver);
            } else {
                strip_comments(path, saver);
            }
        }
        Err(e) => {
            println!("An error occurred: {}", e);
        }
    }
}

fn main() {
    for arg in env::args() {

    }
}
