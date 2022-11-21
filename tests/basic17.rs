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

// This test ensure we don't have an infinite loop in "strip::find_one_of".
const SRC: &str = r#"
pub const MIME_TYPE_JPEG: &str = "image/jpeg";
pub const MIME_TYPE_PNG: &str = "image/png";
pub const MIME_TYPE_JP2: &str = "image/jp2";
pub const MIME_TYPE_URI: &str = "text/x-uri";"#;

#[allow(unused_must_use)]
#[test]
fn strip_ignore() {
    let test_file = "strip.rs";
    let comment_file = "strip.md";
    let temp_dir = tempdir().unwrap();
    gen_file(&temp_dir, test_file, SRC);
    {
        let mut f = gen_file(&temp_dir, comment_file, "");
        stripper_lib::strip_comments(temp_dir.path(), test_file, &mut f, true);
    }
}
