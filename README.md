# rustdoc-stripper

`rustdoc-stripper` is a tool used to remove rustdoc comments from your code and save them in a
`comments.cmts` file if you want to regenerate them.

###Options

Available options for rustdoc-stripper are:

* -h | --help          : Displays this help
* -s | --strip         : Strips the current folder files and create a file with rustdoc information (comments.cmts by default)
* -g | --regenerate    : Recreate files with rustdoc comments from reading rustdoc information file (comments.cmts by default)
* -n | --no-file-output: Display rustdoc information directly on stdout

By default, rustdoc is run with -s option:

```Shell
./rustdoc-stripper -s
```
