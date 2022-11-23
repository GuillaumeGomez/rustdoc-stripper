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

mod common;
use common::*;

const SRC: &str = r#"/// not stripped comment
struct Foo;

impl Foo {
    /// another existing comment
    fn new() -> Foo {}
}

struct Bar;
"#;

const SRC_REGEN: &str = r#"/// not stripped comment
struct Foo;

impl Foo {
    /// another existing comment
    fn new() -> Foo {}
}

/// struct Bar comment
struct Bar;
"#;

fn get_md(file: &str) -> String {
    format!(
        r#"<!-- file {} -->
<!-- struct Foo -->
struct Foo comment
<!-- impl Foo::fn new -->
fn new comment
<!-- struct Bar -->
struct Bar comment
"#,
        file
    )
}

// test if ignore_doc_commented option is working
#[allow(unused_must_use)]
#[test]
fn regeneration() {
    let test_file = "basic.rs";
    let comment_file = "basic.md";
    let temp_dir = tempdir().unwrap();
    gen_file(&temp_dir, test_file, SRC);
    gen_file(&temp_dir, comment_file, &get_md(test_file));
    stripper_lib::regenerate_doc_comments(
        temp_dir.path().to_str().unwrap(),
        false,
        temp_dir.path().join(comment_file).to_str().unwrap(),
        false,
        true,
    );
    compare_files(SRC_REGEN, &temp_dir.path().join(test_file));
}
