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
use tempfile::{tempdir, TempDir};

const BASIC: &str = r#"//! File comment
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

const BASIC_STRIPPED: &str = r#"struct Foo {
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

fn gen_file(temp_dir: &TempDir, filename: &str, content: &str) -> File {
    let mut f = File::create(temp_dir.path().join(filename)).expect("gen_file");
    write!(f, "{}", content).unwrap();
    f
}

fn compare_files(expected_content: &str, file: &Path) {
    let mut f = File::open(file).expect("compare_files '{}'");
    let mut buf = String::new();
    f.read_to_string(&mut buf).unwrap();
    println!();
    for (l, r) in expected_content.lines().zip(buf.lines()) {
        assert_eq!(l, r, "compare_files0 failed");
        println!("{}", l);
    }
    assert_eq!(expected_content, &buf, "compare_files1 failed");
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
    compare_files(
        &get_basic_md(test_file),
        &temp_dir.path().join(comment_file),
    );
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
    stripper_lib::regenerate_doc_comments(
        temp_dir.path().to_str().unwrap(),
        false,
        &temp_dir.path().join(comment_file).to_str().unwrap(),
        false,
        false,
    );
    compare_files(BASIC, &temp_dir.path().join(test_file));
}

const BASIC2: &str = r#"
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

const BASIC2_STRIPPED: &str = r#"
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
    format!(
        r#"<!-- file {} -->
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
"#,
        file
    )
}

const BASIC2_MD: &str = r#"<!-- file * -->
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
    compare_files(
        &get_basic2_md(test_file),
        &temp_dir.path().join(comment_file),
    );
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
    stripper_lib::regenerate_doc_comments(
        temp_dir.path().to_str().unwrap(),
        false,
        &temp_dir.path().join(comment_file).to_str().unwrap(),
        true,
        false,
    );
    compare_files(BASIC2, &temp_dir.path().join(test_file));
}

const BASIC3: &str = r#"///struct Foo comment
struct Foo;
"#;

const BASIC3_STRIPPED: &str = r#"struct Foo;
"#;

const BASIC3_REGEN: &str = r#"/// struct Foo comment
struct Foo;
"#;

fn get_basic3_md(file: &str) -> String {
    format!(
        r#"<!-- file {} -->
<!-- struct Foo -->
struct Foo comment
"#,
        file
    )
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
    compare_files(
        &get_basic3_md(test_file),
        &temp_dir.path().join(comment_file),
    );
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
    stripper_lib::regenerate_doc_comments(
        temp_dir.path().to_str().unwrap(),
        false,
        &temp_dir.path().join(comment_file).to_str().unwrap(),
        false,
        false,
    );
    compare_files(BASIC3_REGEN, &temp_dir.path().join(test_file));
}

const BASIC4: &str = r#"// Copyright 2013-2015, The Gtk-rs Project Developers.
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
    stripper_lib::regenerate_doc_comments(
        temp_dir.path().to_str().unwrap(),
        false,
        &temp_dir.path().join(comment_file).to_str().unwrap(),
        false,
        false,
    );
    compare_files(BASIC4, &temp_dir.path().join(test_file));
}

const BASIC5: &str = r#"/// Here is a flags!
pub flags SomeFlags : u32 {
    /// a const
    const VISIBLE = 1,
    /// another
    const HIDDEN = 2,
}
"#;

const BASIC5_STRIPPED: &str = r#"pub flags SomeFlags : u32 {
    const VISIBLE = 1,
    const HIDDEN = 2,
}
"#;

fn get_basic5_md(file: &str) -> String {
    format!(
        r#"<!-- file {} -->
<!-- flags SomeFlags -->
Here is a flags!
<!-- flags SomeFlags::const VISIBLE -->
a const
<!-- flags SomeFlags::const HIDDEN -->
another
"#,
        file
    )
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
    compare_files(
        &get_basic5_md(test_file),
        &temp_dir.path().join(comment_file),
    );
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
    stripper_lib::regenerate_doc_comments(
        temp_dir.path().to_str().unwrap(),
        false,
        &temp_dir.path().join(comment_file).to_str().unwrap(),
        false,
        false,
    );
    compare_files(BASIC5, &temp_dir.path().join(test_file));
}

const BASIC6: &str = r#"/// not stripped comment
struct Foo;

impl Foo {
    /// another existing comment
    fn new() -> Foo {}
}

struct Bar;
"#;

const BASIC6_REGEN: &str = r#"/// not stripped comment
struct Foo;

impl Foo {
    /// another existing comment
    fn new() -> Foo {}
}

/// struct Bar comment
struct Bar;
"#;

fn get_basic6_md(file: &str) -> String {
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
fn test6_regeneration() {
    let test_file = "basic.rs";
    let comment_file = "basic.md";
    let temp_dir = tempdir().unwrap();
    gen_file(&temp_dir, test_file, BASIC6);
    gen_file(&temp_dir, comment_file, &get_basic6_md(test_file));
    stripper_lib::regenerate_doc_comments(
        temp_dir.path().to_str().unwrap(),
        false,
        &temp_dir.path().join(comment_file).to_str().unwrap(),
        false,
        true,
    );
    compare_files(BASIC6_REGEN, &temp_dir.path().join(test_file));
}

const BASIC7: &str = r#"impl Foo {
    /// existing comment
    pub unsafe fn new() -> Foo {}
}
"#;

fn get_basic7_md(file: &str) -> String {
    format!(
        r#"<!-- file {} -->
<!-- impl Foo::fn new -->
bad comment
"#,
        file
    )
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
    stripper_lib::regenerate_doc_comments(
        temp_dir.path().to_str().unwrap(),
        false,
        &temp_dir.path().join(comment_file).to_str().unwrap(),
        false,
        true,
    );
    compare_files(BASIC7, &temp_dir.path().join(test_file));
}

// The goal of this test is to check if inner macro_rules doc comments are ignored.
const BASIC8: &str = r#"/// foooo
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

const BASIC8_STRIPPED: &str = r#"macro_rules! some_macro {
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

fn get_basic8_md(_file: &str) -> String {
    "<!-- file basic.rs -->\n<!-- macro some_macro -->\nfoooo\n".to_owned()
}

#[allow(unused_must_use)]
#[test]
fn test8_strip() {
    let test_file = "basic.rs";
    let comment_file = "basic.md";
    let temp_dir = tempdir().unwrap();
    gen_file(&temp_dir, test_file, BASIC8);
    {
        let mut f = gen_file(&temp_dir, comment_file, "");
        stripper_lib::strip_comments(temp_dir.path(), test_file, &mut f, false);
    }
    compare_files(
        &get_basic8_md(test_file),
        &temp_dir.path().join(comment_file),
    );
    compare_files(BASIC8_STRIPPED, &temp_dir.path().join(test_file));
}

#[allow(unused_must_use)]
#[test]
fn test8_regeneration() {
    let test_file = "basic.rs";
    let comment_file = "basic.md";
    let temp_dir = tempdir().unwrap();
    gen_file(&temp_dir, test_file, BASIC8);
    gen_file(&temp_dir, comment_file, &get_basic8_md(test_file));
    stripper_lib::regenerate_doc_comments(
        temp_dir.path().to_str().unwrap(),
        false,
        &temp_dir.path().join(comment_file).to_str().unwrap(),
        false,
        true,
    );
    compare_files(BASIC8, &temp_dir.path().join(test_file));
}

const BASIC9: &str = r#"trait SettingsBackendExt: 'static {
    /// Signals that the writability of all keys below a given path.
    pub fn path_writable_changed() {}
}
"#;

const BASIC9_STRIPPED: &str = r#"trait SettingsBackendExt: 'static {
    pub fn path_writable_changed() {}
}
"#;

fn get_basic9_md(file: &str) -> String {
    format!(
        r#"<!-- file {} -->
<!-- trait SettingsBackendExt::fn path_writable_changed -->
Signals that the writability of all keys below a given path.
"#,
        file
    )
}

#[allow(unused_must_use)]
#[test]
fn test9_strip() {
    let test_file = "basic.rs";
    let comment_file = "basic.md";
    let temp_dir = tempdir().unwrap();
    gen_file(&temp_dir, test_file, BASIC9);
    {
        let mut f = gen_file(&temp_dir, comment_file, "");
        stripper_lib::strip_comments(temp_dir.path(), test_file, &mut f, false);
    }
    println!("Testing markdown");
    compare_files(
        &get_basic9_md(test_file),
        &temp_dir.path().join(comment_file),
    );
    println!("Testing stripped file");
    compare_files(BASIC9_STRIPPED, &temp_dir.path().join(test_file));
}

#[allow(unused_must_use)]
#[test]
fn test9_regeneration() {
    let test_file = "basic.rs";
    let comment_file = "basic.md";
    let temp_dir = tempdir().unwrap();
    gen_file(&temp_dir, test_file, BASIC9_STRIPPED);
    gen_file(&temp_dir, comment_file, &get_basic9_md(test_file));
    stripper_lib::regenerate_doc_comments(
        temp_dir.path().to_str().unwrap(),
        false,
        &temp_dir.path().join(comment_file).to_str().unwrap(),
        false,
        true,
    );
    compare_files(BASIC9, &temp_dir.path().join(test_file));
}

const BASIC10: &str = r#"// This file was generated by gir (https://github.com/gtk-rs/gir)
impl Device {
    /// Determines information about the current keyboard grab.
    /// This is not public API and must not be used by applications.
    ///
    /// # Deprecated since 3.16
    ///
    /// The symbol was never meant to be used outside
    ///  of GTK+
    /// ## `display`
    /// the display for which to get the grab information
    /// ## `device`
    /// device to get the grab information from
    /// ## `grab_window`
    /// location to store current grab window
    /// ## `owner_events`
    /// location to store boolean indicating whether
    ///  the `owner_events` flag to `gdk_keyboard_grab` or
    ///  `gdk_pointer_grab` was `true`.
    ///
    /// # Returns
    ///
    /// `true` if this application currently has the
    ///  keyboard grabbed.
    #[cfg_attr(feature = "v3_16", deprecated)]
    pub fn grab_info_libgtk_only(display: &Display, device: &Device) -> Option<(Window, bool)> {
        skip_assert_initialized!();
        unsafe {
            let mut grab_window = ptr::null_mut();
            let mut owner_events = mem::uninitialized();
            let ret = from_glib(gdk_sys::gdk_device_grab_info_libgtk_only(
                display.to_glib_none().0,
                device.to_glib_none().0,
                &mut grab_window,
                &mut owner_events,
            ));
            if ret {
                Some((from_glib_none(grab_window), from_glib(owner_events)))
            } else {
                None
            }
        }
    }
    /// The ::changed signal is emitted either when the `Device`
    /// has changed the number of either axes or keys. For example
    /// In X this will normally happen when the slave device routing
    /// events through the master device changes (for example, user
    /// switches from the USB mouse to a tablet), in that case the
    /// master device will change to reflect the new slave device
    /// axes and keys.
    pub fn connect_changed<F: Fn(&Device) + 'static>(&self, f: F) -> SignalHandlerId {
        unsafe extern "C" fn changed_trampoline<F: Fn(&Device) + 'static>(
            this: *mut gdk_sys::GdkDevice,
            f: glib_sys::gpointer,
        ) {
            let f: &F = &*(f as *const F);
            f(&from_glib_borrow(this))
        }
        unsafe {
            let f: Box_<F> = Box_::new(f);
            connect_raw(
                self.as_ptr() as *mut _,
                b"changed\0".as_ptr() as *const _,
                Some(transmute(changed_trampoline::<F> as usize)),
                Box_::into_raw(f),
            )
        }
    }
}
"#;

const BASIC10_STRIPPED: &str = r#"// This file was generated by gir (https://github.com/gtk-rs/gir)
impl Device {
    #[cfg_attr(feature = "v3_16", deprecated)]
    pub fn grab_info_libgtk_only(display: &Display, device: &Device) -> Option<(Window, bool)> {
        skip_assert_initialized!();
        unsafe {
            let mut grab_window = ptr::null_mut();
            let mut owner_events = mem::uninitialized();
            let ret = from_glib(gdk_sys::gdk_device_grab_info_libgtk_only(
                display.to_glib_none().0,
                device.to_glib_none().0,
                &mut grab_window,
                &mut owner_events,
            ));
            if ret {
                Some((from_glib_none(grab_window), from_glib(owner_events)))
            } else {
                None
            }
        }
    }
    pub fn connect_changed<F: Fn(&Device) + 'static>(&self, f: F) -> SignalHandlerId {
        unsafe extern "C" fn changed_trampoline<F: Fn(&Device) + 'static>(
            this: *mut gdk_sys::GdkDevice,
            f: glib_sys::gpointer,
        ) {
            let f: &F = &*(f as *const F);
            f(&from_glib_borrow(this))
        }
        unsafe {
            let f: Box_<F> = Box_::new(f);
            connect_raw(
                self.as_ptr() as *mut _,
                b"changed\0".as_ptr() as *const _,
                Some(transmute(changed_trampoline::<F> as usize)),
                Box_::into_raw(f),
            )
        }
    }
}
"#;

fn get_basic10_md(file: &str) -> String {
    let x = r###"
<!-- impl Device::fn grab_info_libgtk_only -->
Determines information about the current keyboard grab.
This is not public API and must not be used by applications.

# Deprecated since 3.16

The symbol was never meant to be used outside
 of GTK+
## `display`
the display for which to get the grab information
## `device`
device to get the grab information from
## `grab_window`
location to store current grab window
## `owner_events`
location to store boolean indicating whether
 the `owner_events` flag to `gdk_keyboard_grab` or
 `gdk_pointer_grab` was `true`.

# Returns

`true` if this application currently has the
 keyboard grabbed.
<!-- impl Device::fn connect_changed -->
The ::changed signal is emitted either when the `Device`
has changed the number of either axes or keys. For example
In X this will normally happen when the slave device routing
events through the master device changes (for example, user
switches from the USB mouse to a tablet), in that case the
master device will change to reflect the new slave device
axes and keys.
"###;
    let mut y = format!("<!-- file {} -->", file);
    y.push_str(x);
    y
}

#[allow(unused_must_use)]
#[test]
fn test10_strip() {
    let test_file = "basic.rs";
    let comment_file = "basic.md";
    let temp_dir = tempdir().unwrap();
    gen_file(&temp_dir, test_file, BASIC10);
    {
        let mut f = gen_file(&temp_dir, comment_file, "");
        stripper_lib::strip_comments(temp_dir.path(), test_file, &mut f, false);
    }
    println!("Testing markdown");
    compare_files(
        &get_basic10_md(test_file),
        &temp_dir.path().join(comment_file),
    );
    println!("Testing stripped file");
    compare_files(BASIC10_STRIPPED, &temp_dir.path().join(test_file));
}

#[allow(unused_must_use)]
#[test]
fn test10_regeneration() {
    let test_file = "basic.rs";
    let comment_file = "basic.md";
    let temp_dir = tempdir().unwrap();
    gen_file(&temp_dir, test_file, BASIC10_STRIPPED);
    gen_file(&temp_dir, comment_file, &get_basic10_md(test_file));
    stripper_lib::regenerate_doc_comments(
        temp_dir.path().to_str().unwrap(),
        false,
        &temp_dir.path().join(comment_file).to_str().unwrap(),
        true,
        false,
    );
    compare_files(BASIC10, &temp_dir.path().join(test_file));
}

#[allow(unused_must_use)]
#[test]
fn test10_regeneration2() {
    let test_file = "basic.rs";
    let comment_file = "basic.md";
    let temp_dir = tempdir().unwrap();
    gen_file(&temp_dir, test_file, BASIC10_STRIPPED);
    gen_file(&temp_dir, comment_file, &get_basic10_md(test_file));
    stripper_lib::regenerate_doc_comments(
        temp_dir.path().to_str().unwrap(),
        false,
        &temp_dir.path().join(comment_file).to_str().unwrap(),
        false,
        true,
    );
    compare_files(BASIC10, &temp_dir.path().join(test_file));
}

#[allow(unused_must_use)]
#[test]
fn test10_regeneration3() {
    let test_file = "basic.rs";
    let comment_file = "basic.md";
    let temp_dir = tempdir().unwrap();
    gen_file(&temp_dir, test_file, BASIC10_STRIPPED);
    gen_file(&temp_dir, comment_file, &get_basic10_md(test_file));
    stripper_lib::regenerate_doc_comments(
        temp_dir.path().to_str().unwrap(),
        false,
        &temp_dir.path().join(comment_file).to_str().unwrap(),
        false,
        false,
    );
    compare_files(BASIC10, &temp_dir.path().join(test_file));
}

#[allow(unused_must_use)]
#[test]
fn test10_regeneration4() {
    let test_file = "basic.rs";
    let comment_file = "basic.md";
    let temp_dir = tempdir().unwrap();
    gen_file(&temp_dir, test_file, BASIC10_STRIPPED);
    gen_file(&temp_dir, comment_file, &get_basic10_md(test_file));
    stripper_lib::regenerate_doc_comments(
        temp_dir.path().to_str().unwrap(),
        false,
        &temp_dir.path().join(comment_file).to_str().unwrap(),
        true,
        true,
    );
    compare_files(BASIC10, &temp_dir.path().join(test_file));
}

const BASIC11: &str = r#"// This file was generated by gir (https://github.com/gtk-rs/gir)
impl Device {
    /// The ::changed signal is emitted either when the `Device`
    /// has changed the number of either axes or keys. For example
    /// In X this will normally happen when the slave device routing
    /// events through the master device changes (for example, user
    /// switches from the USB mouse to a tablet), in that case the
    /// master device will change to reflect the new slave device
    /// axes and keys.
    pub fn connect_changed<F: Fn(&Device) + 'static>(&self, f: F) -> SignalHandlerId {
        unsafe extern "C" fn changed_trampoline<F: Fn(&Device) + 'static>(
            this: *mut gdk_sys::GdkDevice,
            f: glib_sys::gpointer,
        ) {
            let f: &F = &*(f as *const F);
            f(&from_glib_borrow(this))
        }
        unsafe {
            let f: Box_<F> = Box_::new(f);
            connect_raw(
                self.as_ptr() as *mut _,
                b"changed\0".as_ptr() as *const _,
                Some(transmute(changed_trampoline::<F> as usize)),
                Box_::into_raw(f),
            )
        }
    }
    /// The ::tool-changed signal is emitted on pen/eraser
    /// ``GdkDevices`` whenever tools enter or leave proximity.
    ///
    /// Feature: `v3_22`
    ///
    /// ## `tool`
    /// The new current tool
    #[cfg(any(feature = "v3_22", feature = "dox"))]
    pub fn connect_tool_changed<F: Fn(&Device, &DeviceTool) + 'static>(
        &self,
        f: F,
    ) -> SignalHandlerId {
        unsafe extern "C" fn tool_changed_trampoline<F: Fn(&Device, &DeviceTool) + 'static>(
            this: *mut gdk_sys::GdkDevice,
            tool: *mut gdk_sys::GdkDeviceTool,
            f: glib_sys::gpointer,
        ) {
            let f: &F = &*(f as *const F);
            f(&from_glib_borrow(this), &from_glib_borrow(tool))
        }
        unsafe {
            let f: Box_<F> = Box_::new(f);
            connect_raw(
                self.as_ptr() as *mut _,
                b"tool-changed\0".as_ptr() as *const _,
                Some(transmute(tool_changed_trampoline::<F> as usize)),
                Box_::into_raw(f),
            )
        }
    }
}
"#;

const BASIC11_STRIPPED: &str = r#"// This file was generated by gir (https://github.com/gtk-rs/gir)
impl Device {
    pub fn connect_changed<F: Fn(&Device) + 'static>(&self, f: F) -> SignalHandlerId {
        unsafe extern "C" fn changed_trampoline<F: Fn(&Device) + 'static>(
            this: *mut gdk_sys::GdkDevice,
            f: glib_sys::gpointer,
        ) {
            let f: &F = &*(f as *const F);
            f(&from_glib_borrow(this))
        }
        unsafe {
            let f: Box_<F> = Box_::new(f);
            connect_raw(
                self.as_ptr() as *mut _,
                b"changed\0".as_ptr() as *const _,
                Some(transmute(changed_trampoline::<F> as usize)),
                Box_::into_raw(f),
            )
        }
    }
    #[cfg(any(feature = "v3_22", feature = "dox"))]
    pub fn connect_tool_changed<F: Fn(&Device, &DeviceTool) + 'static>(
        &self,
        f: F,
    ) -> SignalHandlerId {
        unsafe extern "C" fn tool_changed_trampoline<F: Fn(&Device, &DeviceTool) + 'static>(
            this: *mut gdk_sys::GdkDevice,
            tool: *mut gdk_sys::GdkDeviceTool,
            f: glib_sys::gpointer,
        ) {
            let f: &F = &*(f as *const F);
            f(&from_glib_borrow(this), &from_glib_borrow(tool))
        }
        unsafe {
            let f: Box_<F> = Box_::new(f);
            connect_raw(
                self.as_ptr() as *mut _,
                b"tool-changed\0".as_ptr() as *const _,
                Some(transmute(tool_changed_trampoline::<F> as usize)),
                Box_::into_raw(f),
            )
        }
    }
}
"#;

fn get_basic11_md(file: &str) -> String {
    let x = r###"
<!-- impl Device::fn connect_changed -->
The ::changed signal is emitted either when the `Device`
has changed the number of either axes or keys. For example
In X this will normally happen when the slave device routing
events through the master device changes (for example, user
switches from the USB mouse to a tablet), in that case the
master device will change to reflect the new slave device
axes and keys.
<!-- impl Device::fn connect_tool_changed -->
The ::tool-changed signal is emitted on pen/eraser
``GdkDevices`` whenever tools enter or leave proximity.

Feature: `v3_22`

## `tool`
The new current tool
"###;
    let mut y = format!("<!-- file {} -->", file);
    y.push_str(x);
    y
}

#[allow(unused_must_use)]
#[test]
fn test11_strip() {
    let test_file = "basic.rs";
    let comment_file = "basic.md";
    let temp_dir = tempdir().unwrap();
    gen_file(&temp_dir, test_file, BASIC11);
    {
        let mut f = gen_file(&temp_dir, comment_file, "");
        stripper_lib::strip_comments(temp_dir.path(), test_file, &mut f, false);
    }
    println!("Testing markdown");
    compare_files(
        &get_basic11_md(test_file),
        &temp_dir.path().join(comment_file),
    );
    println!("Testing stripped file");
    compare_files(BASIC11_STRIPPED, &temp_dir.path().join(test_file));
}

#[allow(unused_must_use)]
#[test]
fn test11_regeneration() {
    let test_file = "basic.rs";
    let comment_file = "basic.md";
    let temp_dir = tempdir().unwrap();
    gen_file(&temp_dir, test_file, BASIC11_STRIPPED);
    gen_file(&temp_dir, comment_file, &get_basic11_md(test_file));
    stripper_lib::regenerate_doc_comments(
        temp_dir.path().to_str().unwrap(),
        false,
        &temp_dir.path().join(comment_file).to_str().unwrap(),
        true,
        false,
    );
    compare_files(BASIC11, &temp_dir.path().join(test_file));
}

const BASIC12: &str = r#"impl Foo {
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

fn get_basic12_md(_file: &str) -> String {
    String::new()
}

// test if ignore_doc_commented option is working
#[allow(unused_must_use)]
#[test]
fn test12_strip() {
    let test_file = "basic.rs";
    let comment_file = "basic.md";
    let temp_dir = tempdir().unwrap();
    gen_file(&temp_dir, test_file, BASIC12);
    {
        let mut f = gen_file(&temp_dir, comment_file, "");
        stripper_lib::strip_comments(temp_dir.path(), test_file, &mut f, false);
    }
    println!("Testing markdown");
    compare_files(
        &get_basic12_md(test_file),
        &temp_dir.path().join(comment_file),
    );
    println!("Testing stripped file");
    compare_files(BASIC12, &temp_dir.path().join(test_file));
}

const BASIC13: &str = r#"mod bar {
    // rustdoc-stripper-ignore-next
    /*! Fine

    and you? */
}

mod foobar {
    //! hard day...
}
"#;

const BASIC13_WEIRD: &str = r#"mod bar {
    // rustdoc-stripper-ignore-next
    /*! Fine

    and you? */
}

/// hard day...
mod foobar {
}
"#;

const BASIC13_STRIPPED: &str = r#"mod bar {
    // rustdoc-stripper-ignore-next
    /*! Fine

    and you? */
}

mod foobar {
}
"#;

fn get_basic13_md(file: &str) -> String {
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
    gen_file(&temp_dir, test_file, BASIC13);
    {
        let mut f = gen_file(&temp_dir, comment_file, "");
        stripper_lib::strip_comments(temp_dir.path(), test_file, &mut f, false);
    }
    println!("Testing markdown");
    compare_files(
        &get_basic13_md(test_file),
        &temp_dir.path().join(comment_file),
    );
    println!("Testing stripped file");
    compare_files(BASIC13_STRIPPED, &temp_dir.path().join(test_file));
}

#[allow(unused_must_use)]
#[test]
fn test13_regeneration() {
    let test_file = "basic.rs";
    let comment_file = "basic.md";
    let temp_dir = tempdir().unwrap();
    gen_file(&temp_dir, test_file, BASIC13_STRIPPED);
    gen_file(&temp_dir, comment_file, &get_basic13_md(test_file));
    stripper_lib::regenerate_doc_comments(
        temp_dir.path().to_str().unwrap(),
        false,
        &temp_dir.path().join(comment_file).to_str().unwrap(),
        false,
        false,
    );
    compare_files(BASIC13_WEIRD, &temp_dir.path().join(test_file));
}

const BASIC14: &str = r#"
/// Foo
enum Foo {
    /// Bar
    Bar,
    Blabla,
    /// Toto
    Toto,
}
"#;

const BASIC14_STRIPPED: &str = r#"
enum Foo {
    Bar,
    Blabla,
    Toto,
}
"#;

fn get_basic14_md(file: &str) -> String {
    format!(
        r#"<!-- file {} -->
<!-- enum Foo -->
Foo
<!-- enum Foo::variant Bar -->
Bar
<!-- enum Foo::variant Toto -->
Toto
"#,
        file
    )
}

const BASIC14_MD: &str = r#"<!-- file * -->
<!-- enum Foo -->
Foo
<!-- enum Foo::variant Bar -->
Bar
<!-- enum Foo::variant Toto -->
Toto
"#;

#[allow(unused_must_use)]
#[test]
fn test14_strip_enum() {
    let test_file = "basic14.rs";
    let comment_file = "basic14.md";
    let temp_dir = tempdir().unwrap();
    gen_file(&temp_dir, test_file, BASIC14);
    {
        let mut f = gen_file(&temp_dir, comment_file, "");
        stripper_lib::strip_comments(temp_dir.path(), test_file, &mut f, true);
    }
    compare_files(
        &get_basic14_md(test_file),
        &temp_dir.path().join(comment_file),
    );
    compare_files(BASIC14_STRIPPED, &temp_dir.path().join(test_file));
}

#[allow(unused_must_use)]
#[test]
fn test14_regeneration_enum() {
    let test_file = "basic14.rs";
    let comment_file = "basic14.md";
    let temp_dir = tempdir().unwrap();
    gen_file(&temp_dir, test_file, BASIC14_STRIPPED);
    gen_file(&temp_dir, comment_file, BASIC14_MD);
    stripper_lib::regenerate_doc_comments(
        temp_dir.path().to_str().unwrap(),
        false,
        &temp_dir.path().join(comment_file).to_str().unwrap(),
        true,
        false,
    );
    compare_files(BASIC14, &temp_dir.path().join(test_file));
}

const BASIC15_STRIPPED: &str = r#"
// rustdoc-stripper-ignore-next
/// Calls `gtk_widget_destroy()` on this widget.
///
/// # Safety
///
/// This will not necessarily entirely remove the widget from existence but
/// you must *NOT* query the widget's state subsequently.  Do not call this
/// yourself unless you really mean to.
pub fn foo() {}

// rustdoc-stripper-ignore-next
/// lol
pub fn bar() {}
"#;

const BASIC15: &str = r#"
// rustdoc-stripper-ignore-next
/// Calls `gtk_widget_destroy()` on this widget.
///
/// # Safety
///
/// This will not necessarily entirely remove the widget from existence but
/// you must *NOT* query the widget's state subsequently.  Do not call this
/// yourself unless you really mean to.
// rustdoc-stripper-ignore-next-stop
/// Utility function; intended to be connected to the `Widget::delete-event`
/// signal on a `Window`. The function calls `WidgetExt::hide` on its
/// argument, then returns `true`. If connected to ::delete-event, the
/// result is that clicking the close button for a window (on the
/// window frame, top right corner usually) will hide but not destroy
/// the window. By default, GTK+ destroys windows when ::delete-event
/// is received.
///
/// # Returns
///
/// `true`
pub fn foo() {}

// rustdoc-stripper-ignore-next
/// lol
pub fn bar() {}
"#;

const BASIC15_MD: &str = r#"<!-- fn foo -->
Utility function; intended to be connected to the `Widget::delete-event`
signal on a `Window`. The function calls `WidgetExt::hide` on its
argument, then returns `true`. If connected to ::delete-event, the
result is that clicking the close button for a window (on the
window frame, top right corner usually) will hide but not destroy
the window. By default, GTK+ destroys windows when ::delete-event
is received.

# Returns

`true`
"#;

fn get_basic15_md(file: &str) -> String {
    format!(
        r#"<!-- file {} -->
{}"#,
        file, BASIC15_MD,
    )
}

#[allow(unused_must_use)]
#[test]
fn test15_regeneration_ignore() {
    let test_file = "basic15.rs";
    let comment_file = "basic15.md";
    let temp_dir = tempdir().unwrap();
    gen_file(&temp_dir, test_file, BASIC15_STRIPPED);
    gen_file(&temp_dir, comment_file, &get_basic15_md(test_file));
    stripper_lib::regenerate_doc_comments(
        temp_dir.path().to_str().unwrap(),
        false,
        &temp_dir.path().join(comment_file).to_str().unwrap(),
        true,
        false,
    );
    compare_files(BASIC15, &temp_dir.path().join(test_file));
}

#[allow(unused_must_use)]
#[test]
fn test15_strip_ignore() {
    let test_file = "basic15-strip.rs";
    let comment_file = "basic15-strip.md";
    let temp_dir = tempdir().unwrap();
    gen_file(&temp_dir, test_file, BASIC15);
    {
        let mut f = gen_file(&temp_dir, comment_file, "");
        stripper_lib::strip_comments(temp_dir.path(), test_file, &mut f, true);
    }
    compare_files(
        &get_basic15_md(test_file),
        &temp_dir.path().join(comment_file),
    );
    compare_files(BASIC15_STRIPPED, &temp_dir.path().join(test_file));
}

const BASIC16: &str = r#"impl Pixbuf {
    pub fn from_mut_slice<T: AsMut<[u8]>>() -> Pixbuf {
        let last_row_len = width * ((n_channels * bits_per_sample + 7) / 8);
    }

    // rustdoc-stripper-ignore-next
    /// Creates a `Pixbuf` from a type implementing `Read` (like `File`).
    ///
    /// ```no_run
    /// use std::fs::File;
    /// use gdk_pixbuf::Pixbuf;
    ///
    /// let f = File::open("some_file.png").expect("failed to open image");
    /// let pixbuf = Pixbuf::from_read(f).expect("failed to load image");
    /// ```
    pub fn from_read<R: Read + Send + 'static>(r: R) -> Result<Pixbuf, Error> {
        Pixbuf::from_stream(&gio::ReadInputStream::new(r), None::<&gio::Cancellable>)
    }
}"#;

#[allow(unused_must_use)]
#[test]
fn test16_strip_ignore() {
    let test_file = "basic16-strip.rs";
    let comment_file = "basic16-strip.md";
    let temp_dir = tempdir().unwrap();
    gen_file(&temp_dir, test_file, BASIC16);
    {
        let mut f = gen_file(&temp_dir, comment_file, "");
        stripper_lib::strip_comments(temp_dir.path(), test_file, &mut f, true);
    }
    compare_files(BASIC16, &temp_dir.path().join(test_file));
}

// This test ensure we don't have an infinite loop in "strip::find_one_of".
const BASIC17: &str = r#"
pub const MIME_TYPE_JPEG: &str = "image/jpeg";
pub const MIME_TYPE_PNG: &str = "image/png";
pub const MIME_TYPE_JP2: &str = "image/jp2";
pub const MIME_TYPE_URI: &str = "text/x-uri";"#;

#[allow(unused_must_use)]
#[test]
fn test17_strip_ignore() {
    let test_file = "basic17-strip.rs";
    let comment_file = "basic17-strip.md";
    let temp_dir = tempdir().unwrap();
    gen_file(&temp_dir, test_file, BASIC17);
    {
        let mut f = gen_file(&temp_dir, comment_file, "");
        stripper_lib::strip_comments(temp_dir.path(), test_file, &mut f, true);
    }
}
