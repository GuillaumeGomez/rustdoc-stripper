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

static SRC_RS: &str = r###"
    glib::wrapper! {
        pub struct BaseInfo: u32 {
            const READABLE = 5
        };
    }
"###;

static DOCS_MD: &str = r###"<!-- file * -->
<!-- struct BaseInfo -->
GIBaseInfo is the common base struct of all other Info structs
accessible through the [`Repository`][crate::Repository] API.
<!-- struct BaseInfo::const READABLE -->
This thing can be read.
"###;

static TARGET_RS: &str = r###"
    glib::wrapper! {
        /// GIBaseInfo is the common base struct of all other Info structs
        /// accessible through the [`Repository`][crate::Repository] API.
        pub struct BaseInfo: u32 {
            /// This thing can be read.
            const READABLE = 5
        };
    }
"###;


#[test]
fn item_inside_macro() {
    let src_path = "src.rs";
    let docs_path = "docs.md";
    let temp_dir = tempdir().unwrap();
    gen_file(&temp_dir, src_path, SRC_RS);
    gen_file(&temp_dir, docs_path, DOCS_MD);

    stripper_lib::regenerate_doc_comments(
        temp_dir.path().to_str().unwrap(),
        false,
        temp_dir.path().join(docs_path).to_str().unwrap(),
        false,
        false,
    );
    compare_files(TARGET_RS, &temp_dir.path().join(src_path));
}
