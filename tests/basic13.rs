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

const SRC: &str = r#"mod bar {
    // rustdoc-stripper-ignore-next
    /*! Fine

    and you? */
}

mod foobar {
    //! hard day...
}
"#;

const SRC_WEIRD: &str = r#"mod bar {
    // rustdoc-stripper-ignore-next
    /*! Fine

    and you? */
}

/// hard day...
mod foobar {
}
"#;

const SRC_STRIPPED: &str = r#"mod bar {
    // rustdoc-stripper-ignore-next
    /*! Fine

    and you? */
}

mod foobar {
}
"#;

fn get_md(file: &str) -> String {
    let x = r###"
<!-- file_comment mod foobar -->
hard day...
"###;
    let mut y = format!("<!-- file {} -->", file);
    y.push_str(x);
    y
}

#[allow(unused_must_use)]
#[test]
fn test13_strip() {
    let test_file = "basic.rs";
    let comment_file = "basic.md";
    let temp_dir = tempdir().unwrap();
    gen_file(&temp_dir, test_file, SRC);
    {
        let mut f = gen_file(&temp_dir, comment_file, "");
        stripper_lib::strip_comments(temp_dir.path(), test_file, &mut f, false);
    }
    println!("Testing markdown");
    compare_files(&get_md(test_file), &temp_dir.path().join(comment_file));
    println!("Testing stripped file");
    compare_files(SRC_STRIPPED, &temp_dir.path().join(test_file));
}

#[allow(unused_must_use)]
#[test]
fn regeneration() {
    let test_file = "basic.rs";
    let comment_file = "basic.md";
    let temp_dir = tempdir().unwrap();
    gen_file(&temp_dir, test_file, SRC_STRIPPED);
    gen_file(&temp_dir, comment_file, &get_md(test_file));
    stripper_lib::regenerate_doc_comments(
        temp_dir.path().to_str().unwrap(),
        false,
        temp_dir.path().join(comment_file).to_str().unwrap(),
        false,
        false,
    );
    compare_files(SRC_WEIRD, &temp_dir.path().join(test_file));
}
