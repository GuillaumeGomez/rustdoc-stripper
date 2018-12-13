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

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use tempfile::{TempDir, tempdir};

const BASIC : &'static str = r#"//! File comment
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

    mod SubBar {
    }
}
"#;

fn get_basic_md(file: &str) -> String {
    format!(r#"<!-- file {} -->
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
"#, file)
}

const BASIC2 : &'static str = r#"
use Bin;
use Box;
use Buildable;
use Container;
use Widget;
use Window;
use ffi;
use glib::object::Downcast;
use glib::object::IsA;
use glib::translate::*;

glib_wrapper! {
    /// Dialog boxes are a convenient way to prompt the user for a small amount
    /// of input, e.g. to display a message, ask a question, or anything else
    /// that does not require extensive effort on the user’s part.
    ///
    /// ```
    /// {
    ///  dialog = gtk_dialog_new_with_buttons ("Message",
    ///                                        parent,
    ///                                        flags,
    ///                                        _("_OK"),
    ///                                        GTK_RESPONSE_NONE,
    ///                                        NULL);
    /// }
    /// ```
    pub struct Dialog(Object<ffi::GtkDialog>): Widget, Container, Bin, Window, Buildable;

    match fn {
        get_type => || ffi::gtk_dialog_get_type(),
    }
}

impl Dialog {
    /// Creates a new dialog box.
    ///
    /// Widgets should not be packed into this `Window`
    /// directly, but into the `vbox` and `action_area`, as described above.
    ///
    /// # Returns
    ///
    /// the new dialog as a `Widget`
    pub fn new() -> Dialog {
        assert_initialized_main_thread!();
        unsafe {
            Widget::from_glib_none(ffi::gtk_dialog_new()).downcast_unchecked()
        }
    }

    //pub fn new_with_buttons<T: IsA<Window>>(title: Option<&str>, parent: Option<&T>, flags: DialogFlags, first_button_text: Option<&str>, : /*Unknown conversion*//*Unimplemented*/Fundamental: VarArgs) -> Dialog {
    //    unsafe { TODO: call ffi::gtk_dialog_new_with_buttons() }
    //}
}

/// Trait containing all `Dialog` methods.
pub trait DialogExt {
    /// Adds an activatable widget to the action area of a `Dialog`,
    /// connecting a signal handler that will emit the `Dialog::response`
    /// signal on the dialog when the widget is activated. The widget is
    /// appended to the end of the dialog’s action area. If you want to add a
    /// non-activatable widget, simply pack it into the `action_area` field
    /// of the `Dialog` struct.
    fn add_action_widget<T: IsA<Widget>>(&self, child: &T, response_id: i32);

    /// Adds a button with the given text
    fn add_button(&self, button_text: &str, response_id: i32) -> Widget;
}
"#;

const BASIC2_STRIPPED : &'static str = r#"
use Bin;
use Box;
use Buildable;
use Container;
use Widget;
use Window;
use ffi;
use glib::object::Downcast;
use glib::object::IsA;
use glib::translate::*;

glib_wrapper! {
    pub struct Dialog(Object<ffi::GtkDialog>): Widget, Container, Bin, Window, Buildable;

    match fn {
        get_type => || ffi::gtk_dialog_get_type(),
    }
}

impl Dialog {
    pub fn new() -> Dialog {
        assert_initialized_main_thread!();
        unsafe {
            Widget::from_glib_none(ffi::gtk_dialog_new()).downcast_unchecked()
        }
    }

    //pub fn new_with_buttons<T: IsA<Window>>(title: Option<&str>, parent: Option<&T>, flags: DialogFlags, first_button_text: Option<&str>, : /*Unknown conversion*//*Unimplemented*/Fundamental: VarArgs) -> Dialog {
    //    unsafe { TODO: call ffi::gtk_dialog_new_with_buttons() }
    //}
}

pub trait DialogExt {
    fn add_action_widget<T: IsA<Widget>>(&self, child: &T, response_id: i32);

    fn add_button(&self, button_text: &str, response_id: i32) -> Widget;
}
"#;

fn get_basic2_md(file: &str) -> String {
    format!(r#"<!-- file {} -->
<!-- struct Dialog -->
Dialog boxes are a convenient way to prompt the user for a small amount
of input, e.g. to display a message, ask a question, or anything else
that does not require extensive effort on the user’s part.

```
{{
 dialog = gtk_dialog_new_with_buttons ("Message",
                                       parent,
                                       flags,
                                       _("_OK"),
                                       GTK_RESPONSE_NONE,
                                       NULL);
}}
```
<!-- impl Dialog::fn new -->
Creates a new dialog box.

Widgets should not be packed into this `Window`
directly, but into the `vbox` and `action_area`, as described above.

# Returns

the new dialog as a `Widget`
<!-- trait DialogExt -->
Trait containing all `Dialog` methods.
<!-- trait DialogExt::fn add_action_widget -->
Adds an activatable widget to the action area of a `Dialog`,
connecting a signal handler that will emit the `Dialog::response`
signal on the dialog when the widget is activated. The widget is
appended to the end of the dialog’s action area. If you want to add a
non-activatable widget, simply pack it into the `action_area` field
of the `Dialog` struct.
<!-- trait DialogExt::fn add_button -->
Adds a button with the given text
"#, file)
}

const BASIC2_MD: &'static str = r#"<!-- file * -->
<!-- struct Dialog -->
Dialog boxes are a convenient way to prompt the user for a small amount
of input, e.g. to display a message, ask a question, or anything else
that does not require extensive effort on the user’s part.

```
{
 dialog = gtk_dialog_new_with_buttons ("Message",
                                       parent,
                                       flags,
                                       _("_OK"),
                                       GTK_RESPONSE_NONE,
                                       NULL);
}
```
<!-- impl Dialog::fn new -->
Creates a new dialog box.

Widgets should not be packed into this `Window`
directly, but into the `vbox` and `action_area`, as described above.

# Returns

the new dialog as a `Widget`
<!-- impl Dialog::fn new_with_buttons -->
Creates a new `Dialog` with title `title` (or `None` for the default
title; see `Window::set_title`) and transient parent `parent` (or
`None` for none; see `Window::set_transient_for`).
<!-- trait DialogExt -->
Trait containing all `Dialog` methods.
<!-- trait DialogExt::fn add_action_widget -->
Adds an activatable widget to the action area of a `Dialog`,
connecting a signal handler that will emit the `Dialog::response`
signal on the dialog when the widget is activated. The widget is
appended to the end of the dialog’s action area. If you want to add a
non-activatable widget, simply pack it into the `action_area` field
of the `Dialog` struct.
<!-- trait DialogExt::fn add_button -->
Adds a button with the given text
"#;

const BASIC3 : &'static str = r#"///struct Foo comment
struct Foo;
"#;

const BASIC3_STRIPPED : &'static str = r#"struct Foo;
"#;

const BASIC3_REGEN : &'static str = r#"/// struct Foo comment
struct Foo;
"#;

fn get_basic3_md(file: &str) -> String {
    format!(r#"<!-- file {} -->
<!-- struct Foo -->
struct Foo comment
"#, file)
}

const BASIC4 : &'static str = r#"// Copyright 2013-2015, The Gtk-rs Project Developers.
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

fn get_basic4_md() -> String {
    String::new()
}

const BASIC5 : &'static str = r#"/// Here is a flags!
pub flags SomeFlags : u32 {
    /// a const
    const VISIBLE = 1,
    /// another
    const HIDDEN = 2,
}
"#;

const BASIC5_STRIPPED : &'static str = r#"pub flags SomeFlags : u32 {
    const VISIBLE = 1,
    const HIDDEN = 2,
}
"#;

fn get_basic5_md(file: &str) -> String {
    format!(r#"<!-- file {} -->
<!-- flags SomeFlags -->
Here is a flags!
<!-- flags SomeFlags::const VISIBLE -->
a const
<!-- flags SomeFlags::const HIDDEN -->
another
"#, file)
}

const BASIC6 : &'static str = r#"/// not stripped comment
struct Foo;

impl Foo {
    /// another existing comment
    fn new() -> Foo {}
}

struct Bar;
"#;

const BASIC6_REGEN : &'static str = r#"/// not stripped comment
struct Foo;

impl Foo {
    /// another existing comment
    fn new() -> Foo {}
}

/// struct Bar comment
struct Bar;
"#;

fn get_basic6_md(file: &str) -> String {
    format!(r#"<!-- file {} -->
<!-- struct Foo -->
struct Foo comment
<!-- impl Foo::fn new -->
fn new comment
<!-- struct Bar -->
struct Bar comment
"#, file)
}

const BASIC7 : &'static str = r#"impl Foo {
    /// existing comment
    pub unsafe fn new() -> Foo {}
}
"#;

fn get_basic7_md(file: &str) -> String {
    format!(r#"<!-- file {} -->
<!-- impl Foo::fn new -->
bad comment
"#, file)
}

fn gen_file(temp_dir: &TempDir, filename: &str, content: &str) -> File {
    let mut f = File::create(temp_dir.path().join(filename)).expect("gen_file");
    write!(f, "{}", content).unwrap();
    f
}

fn compare_files(expected_content: &str, file: &Path) {
    let mut f = File::open(file).expect("compare_files '{}'");
    let mut buf = String::new();
    f.read_to_string(&mut buf).unwrap();
    println!("");
    for (l, r) in expected_content.lines().zip(buf.lines()) {
        assert_eq!(l, r);
        println!("{}", l);
    }
    assert!(expected_content == &buf);
}

#[allow(unused_must_use)]
#[test]
fn test_strip() {
    let test_file = "basic.rs";
    let comment_file = "basic.md";
    let temp_dir = tempdir().unwrap();
    gen_file(&temp_dir, test_file, BASIC);
    {
        let mut f = gen_file(&temp_dir, comment_file, "");
        stripper_lib::strip_comments(temp_dir.path(), test_file, &mut f, false);
    }
    compare_files(&get_basic_md(test_file), &temp_dir.path().join(comment_file));
    compare_files(BASIC_STRIPPED, &temp_dir.path().join(test_file));
}

#[allow(unused_must_use)]
#[test]
fn test_regeneration() {
    let test_file = "basic.rs";
    let comment_file = "basic.md";
    let temp_dir = tempdir().unwrap();
    gen_file(&temp_dir, test_file, BASIC_STRIPPED);
    gen_file(&temp_dir, comment_file, &get_basic_md(test_file));
    stripper_lib::regenerate_doc_comments(temp_dir.path().to_str().unwrap(), false,
                                          &temp_dir.path().join(comment_file).to_str().unwrap(),
                                          false, false);
    compare_files(BASIC, &temp_dir.path().join(test_file));
}

#[allow(unused_must_use)]
#[test]
fn test2_strip() {
    let test_file = "basic.rs";
    let comment_file = "basic.md";
    let temp_dir = tempdir().unwrap();
    gen_file(&temp_dir, test_file, BASIC2);
    {
        let mut f = gen_file(&temp_dir, comment_file, "");
        stripper_lib::strip_comments(temp_dir.path(), test_file, &mut f, true);
    }
    compare_files(&get_basic2_md(test_file), &temp_dir.path().join(comment_file));
    compare_files(BASIC2_STRIPPED, &temp_dir.path().join(test_file));
}

#[allow(unused_must_use)]
#[test]
fn test2_regeneration() {
    let test_file = "basic.rs";
    let comment_file = "basic.md";
    let temp_dir = tempdir().unwrap();
    gen_file(&temp_dir, test_file, BASIC2_STRIPPED);
    gen_file(&temp_dir, comment_file, BASIC2_MD);
    stripper_lib::regenerate_doc_comments(temp_dir.path().to_str().unwrap(), false,
                                          &temp_dir.path().join(comment_file).to_str().unwrap(),
                                          true, false);
    compare_files(BASIC2, &temp_dir.path().join(test_file));
}

#[allow(unused_must_use)]
#[test]
fn test3_strip() {
    let test_file = "basic.rs";
    let comment_file = "basic.md";
    let temp_dir = tempdir().unwrap();
    gen_file(&temp_dir, test_file, BASIC3);
    {
        let mut f = gen_file(&temp_dir, comment_file, "");
        stripper_lib::strip_comments(temp_dir.path(), test_file, &mut f, false);
    }
    compare_files(&get_basic3_md(test_file), &temp_dir.path().join(comment_file));
    compare_files(BASIC3_STRIPPED, &temp_dir.path().join(test_file));
}

#[allow(unused_must_use)]
#[test]
fn test3_regeneration() {
    let test_file = "basic.rs";
    let comment_file = "basic.md";
    let temp_dir = tempdir().unwrap();
    gen_file(&temp_dir, test_file, BASIC3_STRIPPED);
    gen_file(&temp_dir, comment_file, &get_basic3_md(test_file));
    stripper_lib::regenerate_doc_comments(temp_dir.path().to_str().unwrap(), false,
                                          &temp_dir.path().join(comment_file).to_str().unwrap(),
                                          false, false);
    compare_files(BASIC3_REGEN, &temp_dir.path().join(test_file));
}

#[allow(unused_must_use)]
#[test]
fn test4_strip() {
    let test_file = "basic.rs";
    let comment_file = "basic.md";
    let temp_dir = tempdir().unwrap();
    gen_file(&temp_dir, test_file, BASIC4);
    {
        let mut f = gen_file(&temp_dir, comment_file, "");
        stripper_lib::strip_comments(temp_dir.path(), test_file, &mut f, false);
    }
    compare_files(&get_basic4_md(), &temp_dir.path().join(comment_file));
    compare_files(BASIC4, &temp_dir.path().join(test_file));
}

#[allow(unused_must_use)]
#[test]
fn test4_regeneration() {
    let test_file = "basic.rs";
    let comment_file = "basic.md";
    let temp_dir = tempdir().unwrap();
    gen_file(&temp_dir, test_file, BASIC4);
    gen_file(&temp_dir, comment_file, &get_basic4_md());
    stripper_lib::regenerate_doc_comments(temp_dir.path().to_str().unwrap(), false,
                                          &temp_dir.path().join(comment_file).to_str().unwrap(),
                                          false, false);
    compare_files(BASIC4, &temp_dir.path().join(test_file));
}

#[allow(unused_must_use)]
#[test]
fn test5_strip() {
    let test_file = "basic.rs";
    let comment_file = "basic.md";
    let temp_dir = tempdir().unwrap();
    gen_file(&temp_dir, test_file, BASIC5);
    {
        let mut f = gen_file(&temp_dir, comment_file, "");
        stripper_lib::strip_comments(temp_dir.path(), test_file, &mut f, false);
    }
    compare_files(&get_basic5_md(test_file), &temp_dir.path().join(comment_file));
    compare_files(BASIC5_STRIPPED, &temp_dir.path().join(test_file));
}

#[allow(unused_must_use)]
#[test]
fn test5_regeneration() {
    let test_file = "basic.rs";
    let comment_file = "basic.md";
    let temp_dir = tempdir().unwrap();
    gen_file(&temp_dir, test_file, BASIC5_STRIPPED);
    gen_file(&temp_dir, comment_file, &get_basic5_md(test_file));
    stripper_lib::regenerate_doc_comments(temp_dir.path().to_str().unwrap(), false,
                                          &temp_dir.path().join(comment_file).to_str().unwrap(),
                                          false, false);
    compare_files(BASIC5, &temp_dir.path().join(test_file));
}

// test if ignore_doc_commented option is working
#[allow(unused_must_use)]
#[test]
fn test6_regeneration() {
    let test_file = "basic.rs";
    let comment_file = "basic.md";
    let temp_dir = tempdir().unwrap();
    gen_file(&temp_dir, test_file, BASIC6);
    gen_file(&temp_dir, comment_file, &get_basic6_md(test_file));
    stripper_lib::regenerate_doc_comments(temp_dir.path().to_str().unwrap(), false,
                                          &temp_dir.path().join(comment_file).to_str().unwrap(),
                                          false, true);
    compare_files(BASIC6_REGEN, &temp_dir.path().join(test_file));
}

// test if ignore_doc_commented option is working
#[allow(unused_must_use)]
#[test]
fn test7_regeneration() {
    let test_file = "basic.rs";
    let comment_file = "basic.md";
    let temp_dir = tempdir().unwrap();
    gen_file(&temp_dir, test_file, BASIC7);
    gen_file(&temp_dir, comment_file, &get_basic7_md(test_file));
    stripper_lib::regenerate_doc_comments(temp_dir.path().to_str().unwrap(), false,
                                          &temp_dir.path().join(comment_file).to_str().unwrap(),
                                          false, true);
    compare_files(BASIC7, &temp_dir.path().join(test_file));
}
