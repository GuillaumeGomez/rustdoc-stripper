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

const SRC: &str = r#"//! File comment
//! three
//! lines

/// struct Foo comment
struct Foo {
    /// Foo comment
    /// fn some_func(a: u32,
    ///              b: u32) {}
    A: u32,
}

mod Bar {
    //! mod comment
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

    mod SubBar {
        //! an empty mod
        //! yeay
    }
}
"#;

const SRC_STRIPPED: &str = r#"struct Foo {
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

    mod SubBar {
    }
}
"#;

fn get_md(file: &str) -> String {
    format!(
        r#"<!-- file {} -->
<!-- file_comment -->
File comment
three
lines
<!-- struct Foo -->
struct Foo comment
<!-- struct Foo::variant A -->
Foo comment
fn some_func(a: u32,
             b: u32) {{}}
<!-- file_comment mod Bar -->
mod comment
<!-- mod Bar::macro test!::struct SuperFoo -->
struct inside macro
<!-- mod Bar::macro test!::macro sub_test!::struct FooFoo -->
and another one!
<!-- file_comment mod Bar::mod SubBar -->
an empty mod
yeay
"#,
        file
    )
}

#[allow(unused_must_use)]
#[test]
fn strip() {
    let test_file = "basic.rs";
    let comment_file = "basic.md";
    let temp_dir = tempdir().unwrap();
    gen_file(&temp_dir, test_file, SRC);
    {
        let mut f = gen_file(&temp_dir, comment_file, "");
        stripper_lib::strip_comments(temp_dir.path(), test_file, &mut f, false);
    }
    compare_files(&get_md(test_file), &temp_dir.path().join(comment_file));
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
    compare_files(SRC, &temp_dir.path().join(test_file));
}
