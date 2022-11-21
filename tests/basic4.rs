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

const SRC: &str = r#"// Copyright 2013-2015, The Gtk-rs Project Developers.
// See the COPYRIGHT file at the top-level directory of this distribution.
// Licensed under the MIT license, see the LICENSE file or <http://opensource.org/licenses/MIT>

use glib::translate::*;
use ffi;

use glib::object::Downcast;
use Widget;

glib_wrapper! {
    pub struct Socket(Object<ffi::GtkSocket>): Widget, ::Container, ::Buildable;

    match fn {
        get_type => || ffi::gtk_socket_get_type(),
    }
}

impl Socket {
    pub fn new() -> Socket {
        assert_initialized_main_thread!();
        unsafe { Widget::from_glib_none(ffi::gtk_socket_new()).downcast_unchecked() }
    }

    /*pub fn add_id(&self, window: Window) {
        unsafe { ffi::gtk_socket_add_id(self.to_glib_none().0, window) };
    }

    pub fn get_id(&self) -> Window {
        unsafe { ffi::gtk_socket_get_id(self.to_glib_none().0) };
    }

    pub fn get_plug_window(&self) -> GdkWindow {
        let tmp_pointer = unsafe { ffi::gtk_socket_get_plug_window(self.to_glib_none().0) };

        // add end of code
    }*/
}
"#;

fn get_md() -> String {
    String::new()
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
    compare_files(&get_md(), &temp_dir.path().join(comment_file));
    compare_files(SRC, &temp_dir.path().join(test_file));
}

#[allow(unused_must_use)]
#[test]
fn regeneration() {
    let test_file = "basic.rs";
    let comment_file = "basic.md";
    let temp_dir = tempdir().unwrap();
    gen_file(&temp_dir, test_file, SRC);
    gen_file(&temp_dir, comment_file, &get_md());
    stripper_lib::regenerate_doc_comments(
        temp_dir.path().to_str().unwrap(),
        false,
        temp_dir.path().join(comment_file).to_str().unwrap(),
        false,
        false,
    );
    compare_files(SRC, &temp_dir.path().join(test_file));
}
