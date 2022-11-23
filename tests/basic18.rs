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

const SRC: &str = r#"
// Take a look at the license at the top of the repository in the LICENSE file.

use crate::{resources_register, Resource};
use glib::translate::*;
use std::env;
use std::mem;
use std::path::Path;
use std::process::Command;
use std::ptr;

impl Resource {
    #[doc(alias = "g_resource_new_from_data")]
    pub fn from_data(data: &glib::Bytes) -> Result<Resource, glib::Error> {
        unsafe {
            let mut error = ptr::null_mut();

            // Create a copy of data if it is not pointer-aligned
            // https://bugzilla.gnome.org/show_bug.cgi?id=790030
            let mut data = data.clone();
            let data_ptr = glib::ffi::g_bytes_get_data(data.to_glib_none().0, ptr::null_mut());
            if data_ptr as usize % mem::align_of::<*const u8>() != 0 {
                data = glib::Bytes::from(&*data);
            }

            let ret = ffi::g_resource_new_from_data(data.to_glib_none().0, &mut error);
            if error.is_null() {
                Ok(from_glib_full(ret))
            } else {
                Err(from_glib_full(error))
            }
        }
    }
}

/// Call from build script to run `glib-compile-resources` to generate compiled gresources to embed
/// in binary with [resources_register_include]. `target` is relative to `OUT_DIR`.
///
/// ```no_run
/// gio::compile_resources(
///     "resources",
///     "resources/resources.gresource.xml",
///     "compiled.gresource",
/// );
/// ```
pub fn compile_resources<P: AsRef<Path>>(source_dir: P, gresource: &str, target: &str) {
    let out_dir = env::var("OUT_DIR").unwrap();

    let status = Command::new("glib-compile-resources")
        .arg("--sourcedir")
        .arg(source_dir.as_ref())
        .arg("--target")
        .arg(&format!("{}/{}", out_dir, target))
        .arg(gresource)
        .status()
        .unwrap();

    if !status.success() {
        panic!("glib-compile-resources failed with exit status {}", status);
    }

    println!("cargo:rerun-if-changed={}", gresource);
    let output = Command::new("glib-compile-resources")
        .arg("--sourcedir")
        .arg(source_dir.as_ref())
        .arg("--generate-dependencies")
        .arg(gresource)
        .output()
        .unwrap()
        .stdout;
    let output = String::from_utf8(output).unwrap();
    for dep in output.split_whitespace() {
        println!("cargo:rerun-if-changed={}", dep);
    }
}

#[doc(hidden)]
pub fn resources_register_include_impl(bytes: &'static [u8]) -> Result<(), glib::Error> {
    let bytes = glib::Bytes::from_static(bytes);
    let resource = Resource::from_data(&bytes)?;
    resources_register(&resource);
    Ok(())
}

/// Include gresources generated with [compile_resources] and register with glib. `path` is
/// relative to `OUTDIR`.
///
/// ```ignore
/// gio::resources_register_include!("compiled.gresource").unwrap();
/// ```
#[macro_export]
macro_rules! resources_register_include {
    ($path:expr) => {
        $crate::resources_register_include_impl(include_bytes!(concat!(
            env!("OUT_DIR"),
            "/",
            $path
        )))
    };
}
"#;

const SRC_STRIPPED: &str = r#"
// Take a look at the license at the top of the repository in the LICENSE file.

use crate::{resources_register, Resource};
use glib::translate::*;
use std::env;
use std::mem;
use std::path::Path;
use std::process::Command;
use std::ptr;

impl Resource {
    #[doc(alias = "g_resource_new_from_data")]
    pub fn from_data(data: &glib::Bytes) -> Result<Resource, glib::Error> {
        unsafe {
            let mut error = ptr::null_mut();

            // Create a copy of data if it is not pointer-aligned
            // https://bugzilla.gnome.org/show_bug.cgi?id=790030
            let mut data = data.clone();
            let data_ptr = glib::ffi::g_bytes_get_data(data.to_glib_none().0, ptr::null_mut());
            if data_ptr as usize % mem::align_of::<*const u8>() != 0 {
                data = glib::Bytes::from(&*data);
            }

            let ret = ffi::g_resource_new_from_data(data.to_glib_none().0, &mut error);
            if error.is_null() {
                Ok(from_glib_full(ret))
            } else {
                Err(from_glib_full(error))
            }
        }
    }
}

pub fn compile_resources<P: AsRef<Path>>(source_dir: P, gresource: &str, target: &str) {
    let out_dir = env::var("OUT_DIR").unwrap();

    let status = Command::new("glib-compile-resources")
        .arg("--sourcedir")
        .arg(source_dir.as_ref())
        .arg("--target")
        .arg(&format!("{}/{}", out_dir, target))
        .arg(gresource)
        .status()
        .unwrap();

    if !status.success() {
        panic!("glib-compile-resources failed with exit status {}", status);
    }

    println!("cargo:rerun-if-changed={}", gresource);
    let output = Command::new("glib-compile-resources")
        .arg("--sourcedir")
        .arg(source_dir.as_ref())
        .arg("--generate-dependencies")
        .arg(gresource)
        .output()
        .unwrap()
        .stdout;
    let output = String::from_utf8(output).unwrap();
    for dep in output.split_whitespace() {
        println!("cargo:rerun-if-changed={}", dep);
    }
}

#[doc(hidden)]
pub fn resources_register_include_impl(bytes: &'static [u8]) -> Result<(), glib::Error> {
    let bytes = glib::Bytes::from_static(bytes);
    let resource = Resource::from_data(&bytes)?;
    resources_register(&resource);
    Ok(())
}

#[macro_export]
macro_rules! resources_register_include {
    ($path:expr) => {
        $crate::resources_register_include_impl(include_bytes!(concat!(
            env!("OUT_DIR"),
            "/",
            $path
        )))
    };
}
"#;

const MD: &str = r#"<!-- fn compile_resources -->
Call from build script to run `glib-compile-resources` to generate compiled gresources to embed
in binary with [resources_register_include]. `target` is relative to `OUT_DIR`.

```no_run
gio::compile_resources(
    "resources",
    "resources/resources.gresource.xml",
    "compiled.gresource",
);
```
<!-- macro resources_register_include -->
Include gresources generated with [compile_resources] and register with glib. `path` is
relative to `OUTDIR`.

```ignore
gio::resources_register_include!("compiled.gresource").unwrap();
```
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
fn strip_failure() {
    let test_file = "strip.rs";
    let comment_file = "strip.md";
    let temp_dir = tempdir().unwrap();
    gen_file(&temp_dir, test_file, SRC);
    {
        let mut f = gen_file(&temp_dir, comment_file, "");
        stripper_lib::strip_comments(temp_dir.path(), test_file, &mut f, true);
    }
    compare_files(&get_md(test_file), &temp_dir.path().join(comment_file));
    compare_files(SRC_STRIPPED, &temp_dir.path().join(test_file));
}
