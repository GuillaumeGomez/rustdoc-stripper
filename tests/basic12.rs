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

const SRC: &str = r#"impl Foo {
    // rustdoc-stripper-ignore-next
    /// existing comment
    ///
    /// with multiple lines
    pub unsafe fn new() -> Foo {}

    // rustdoc-stripper-ignore-next
    /// one line!
    pub fn bar() {}
}

mod foo {
    // rustdoc-stripper-ignore-next
    //! hello!
    //!
    //! how are you?
}

mod bar {
    // rustdoc-stripper-ignore-next
    /*! Fine

    and you? */
}

mod foobar {
    // rustdoc-stripper-ignore-next
    //! hard day...
}
"#;

fn get_md(_file: &str) -> String {
    String::new()
}

// test if ignore_doc_commented option is working
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
    println!("Testing markdown");
    compare_files(
        &get_md(test_file),
        &temp_dir.path().join(comment_file),
    );
    println!("Testing stripped file");
    compare_files(SRC, &temp_dir.path().join(test_file));
}
