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
use std::collections::HashMap;

use types::{
    TypeStruct,
    EventType,
    Type,
    MOD_COMMENT,
    FILE_COMMENT,
    FILE,
};

/*fn parse_mod_line(line: &str) -> EventType {
    let line = line.replace(MOD_COMMENT, "");
    let parts = line.split("§").collect();

    for part in parts {

    }
}*/

pub fn regenerate_doc_comments() {
    // we start by storing files info
    let mut f = match OpenOptions::new().read(true).open("comments.cmts") {
        Ok(f) => f,
        Err(e) => {
            println!("An error occured while trying to open '{}': {}", "comments.cmts", e);
            return;
        }
    };
    /*let mut reader = BufReader::new(f);
    let mut lines : Vec<String> = vec!();
    let mut current_file = String::new();
    let mut infos = HashMap::new();
    let mut current_infos = vec!();

    for tmp_line in reader.lines() {
        let line = tmp_line.unwrap();
        if line.starts_with(FILE) {
            if current_file.len() > 0 && current_infos.len() > 0 {
                infos.insert(current_file, current_infos.clone());
                current_infos = vec!();
            }
            current_file = line.to_owned();
        } else if line.starts_with(MOD_COMMENT) {
            parse_mod_line(&line);
            // we read as long as we have (mod) comment
        } else if line.starts_with(FILE_COMMENT) {
            // we get after head comment
        }
        lines.push(line.unwrap());
    }
    if current_file.len() > 0 && current_infos.len() > 0 {
        infos.insert(current_file, current_infos.clone());
    }
    run_over_files(&infos);*/
}