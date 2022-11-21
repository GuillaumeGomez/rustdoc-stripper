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

// The goal of this test is to check if inner macro_rules doc comments are ignored.
const SRC: &str = r#"/// foooo
macro_rules! some_macro {
    ($constructor_ffi: ident) => {
        /// Takes full ownership of the output stream,
        /// which is not allowed to borrow any lifetime shorter than `'static`.
        ///
        /// Because the underlying `cairo_surface_t` is reference-counted,
        /// a lifetime parameter in a Rust wrapper type would not be enough to track
        /// how long it can keep writing to the stream.
        pub fn for_stream<W: io::Write + 'static>(width: f64, height: f64, stream: W) -> u32 {
            0
        }
    }
}
"#;

const SRC_STRIPPED: &str = r#"macro_rules! some_macro {
    ($constructor_ffi: ident) => {
        /// Takes full ownership of the output stream,
        /// which is not allowed to borrow any lifetime shorter than `'static`.
        ///
        /// Because the underlying `cairo_surface_t` is reference-counted,
        /// a lifetime parameter in a Rust wrapper type would not be enough to track
        /// how long it can keep writing to the stream.
        pub fn for_stream<W: io::Write + 'static>(width: f64, height: f64, stream: W) -> u32 {
            0
        }
    }
}
"#;

fn get_md(_file: &str) -> String {
    "<!-- file basic.rs -->\n<!-- macro some_macro -->\nfoooo\n".to_owned()
}

#[allow(unused_must_use)]
#[test]
fn test8_strip() {
    let test_file = "basic.rs";
    let comment_file = "basic.md";
    let temp_dir = tempdir().unwrap();
    gen_file(&temp_dir, test_file, SRC);
    {
        let mut f = gen_file(&temp_dir, comment_file, "");
        stripper_lib::strip_comments(temp_dir.path(), test_file, &mut f, false);
    }
    compare_files(
        &get_md(test_file),
        &temp_dir.path().join(comment_file),
    );
    compare_files(SRC_STRIPPED, &temp_dir.path().join(test_file));
}

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
    compare_files(SRC, &temp_dir.path().join(test_file));
}
