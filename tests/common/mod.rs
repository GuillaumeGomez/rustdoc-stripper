// Copyright 2016 Gomez Guillaume
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

extern crate stripper_lib;
extern crate tempfile;

pub use self::tempfile::tempdir;
use self::tempfile::TempDir;
pub use std::fs::File;
pub use std::io::prelude::*;
pub use std::path::Path;

pub fn gen_file(temp_dir: &TempDir, filename: &str, content: &str) -> File {
    let mut f = File::create(temp_dir.path().join(filename)).expect("gen_file");
    write!(f, "{}", content).unwrap();
    f
}

#[allow(dead_code)]
pub fn compare_files(expected_content: &str, file: &Path) {
    let mut f = File::open(file).expect("compare_files '{}'");
    let mut buf = String::new();
    f.read_to_string(&mut buf).unwrap();
    println!();
    for (l, r) in expected_content.lines().zip(buf.lines()) {
        assert_eq!(l, r, "compare_files0 failed");
        println!("{}", l);
    }
    assert_eq!(expected_content, &buf, "compare_files1 failed");
}
