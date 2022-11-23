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

const SRC: &str = r#"impl Pixbuf {
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
fn strip_ignore() {
    let test_file = "strip.rs";
    let comment_file = "strip.md";
    let temp_dir = tempdir().unwrap();
    gen_file(&temp_dir, test_file, SRC);
    {
        let mut f = gen_file(&temp_dir, comment_file, "");
        stripper_lib::strip_comments(temp_dir.path(), test_file, &mut f, true);
    }
    compare_files(SRC, &temp_dir.path().join(test_file));
}
