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
use std::io::{BufRead, BufReader};

pub fn regenerate_doc_comments() {
    // we start by storing files info
    let mut f = match OpenOptions::new().read(true).open("comments.cmts") {
        Ok(f) => {
            f
        }
        Err(e) => {
            println!("An error occured while trying to open '{}': {}", "comments.cmts", e);
            return;
        }
    };
    let mut reader = BufReader::new(f);
    let mut lines : Vec<String> = vec!();

    for line in reader.lines() {
        lines.push(line.unwrap());
    }
}