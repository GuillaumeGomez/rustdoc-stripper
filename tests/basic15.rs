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

const SRC_STRIPPED: &str = r#"
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

const SRC: &str = r#"
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

const MD: &str = r#"<!-- fn foo -->
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

fn get_md(file: &str) -> String {
    format!(
        r#"<!-- file {} -->
{}"#,
        file, MD,
    )
}

#[allow(unused_must_use)]
#[test]
fn regeneration_ignore() {
    let test_file = "basic.rs";
    let comment_file = "basic.md";
    let temp_dir = tempdir().unwrap();
    gen_file(&temp_dir, test_file, SRC_STRIPPED);
    gen_file(&temp_dir, comment_file, &get_md(test_file));
    stripper_lib::regenerate_doc_comments(
        temp_dir.path().to_str().unwrap(),
        false,
        temp_dir.path().join(comment_file).to_str().unwrap(),
        true,
        false,
    );
    compare_files(SRC, &temp_dir.path().join(test_file));
}

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
    compare_files(
        &get_md(test_file),
        &temp_dir.path().join(comment_file),
    );
    compare_files(SRC_STRIPPED, &temp_dir.path().join(test_file));
}
