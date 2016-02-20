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

use std::fs::{File, OpenOptions, create_dir, remove_file};
use std::io::{Read, Write};
use std::path::Path;

extern crate stripper_lib;

const TEST_FILE_DIR : &'static str = "tests/files";

const BASIC : &'static str = r#"/// struct Foo comment
struct Foo {
    /// Foo comment
    /// fn some_func(a: u32,
    ///              b: u32) {}
    A: u32,
}

mod Bar {
    test! {
        /// struct inside macro
        struct SuperFoo;
        sub_test! {
            /// and another one!
            struct FooFoo {
                x: u32,
            }
        }
    }
}
"#;

const BASIC_STRIPPED : &'static str = r#"struct Foo {
    A: u32,
}

mod Bar {
    test! {
        struct SuperFoo;
        sub_test! {
            struct FooFoo {
                x: u32,
            }
        }
    }
}
"#;

fn get_basic_cmt(file: &str) -> String {
    format!(r#"<!-- file {} -->
<!-- struct Foo -->
 struct Foo comment
<!-- struct Foo§variant A -->
 Foo comment
 fn some_func(a: u32,
              b: u32) {}
<!-- mod Bar§macro test!§struct SuperFoo -->
 struct inside macro
<!-- mod Bar§macro test!§macro sub_test!§struct FooFoo -->
 and another one!
"#, file, "{}")
}

fn gen_file(filename: &str, content: &str) -> File {
    match OpenOptions::new().write(true).create(true).truncate(true).open(&format!("{}/{}", TEST_FILE_DIR, filename)) {
        Ok(mut f) => {
            write!(f, "{}", content).unwrap();
            f
        },
        Err(e) => {
            panic!("gen_file: {}", e)
        },
    }
}

fn compare_files(expected_content: &str, file: &str) {
    match File::open(file) {
        Ok(mut f) => {
            let mut buf = String::new();
            f.read_to_string(&mut buf).unwrap();
            assert_eq!(expected_content, &buf);
        },
        Err(e) => panic!("compare_files '{}': {}", file, e),
    }
}

#[allow(unused_must_use)]
fn clean_test(files_to_remove: &[&str]) {
    for file in files_to_remove {
        remove_file(file);
    }
}

#[allow(unused_must_use)]
#[test]
fn test_strip() {
    let test_file = "basic.rs";
    let comment_file = "basic.cmts";
    create_dir(TEST_FILE_DIR);
    {
        gen_file(test_file, BASIC);
        let mut f = gen_file(comment_file, "");
        stripper_lib::strip_comments(Path::new(TEST_FILE_DIR), test_file, &mut f, false);
    }
    compare_files(&get_basic_cmt(test_file), &format!("{}/{}", TEST_FILE_DIR, comment_file));
    compare_files(BASIC_STRIPPED, &format!("{}/{}", TEST_FILE_DIR, test_file));
    clean_test(&vec!(test_file, comment_file));
}

#[allow(unused_must_use)]
#[test]
fn test_regeneration() {
    let test_file = "regen.rs";
    let comment_file = "regen.cmts";
    create_dir(TEST_FILE_DIR);
    {
        gen_file(test_file, BASIC);
        let mut f = gen_file(comment_file, "");
        stripper_lib::strip_comments(Path::new(TEST_FILE_DIR), test_file, &mut f, false);
        stripper_lib::regenerate_doc_comments(TEST_FILE_DIR, false,
                                              &format!("{}/{}", TEST_FILE_DIR, comment_file),
                                              false);
    }
    compare_files(BASIC, &format!("{}/{}", TEST_FILE_DIR, test_file));
    clean_test(&vec!(test_file, comment_file));
}
