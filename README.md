# cargo doc-stripper

This tool is used to move documentation into and out of your source code and
store it into a markdown file (`documentation.md` by default).

To make it work, you need to install both `cargo-doc-stripper` and `cargo-doc-stripper-inner`.
You can then run it as follows:

```
cargo doc-stripper --help
```

If you want to install from this repository, you can do it as follows:

```
cd cargo-doc-stripper
cargo install --path .
cd ../cargo-doc-stripper-inner
cargo install --path .
```
