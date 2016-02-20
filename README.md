# rustdoc-stripper [![Build Status](https://api.travis-ci.org/GuillaumeGomez/rustdoc-stripper.png?branch=master)](https://travis-ci.org/GuillaumeGomez/rustdoc-stripper)

`rustdoc-stripper` is a tool used to remove rustdoc comments from your code and save them in a
`comments.cmts` file if you want to regenerate them.

###Options

Available options for rustdoc-stripper are:

* -h | --help             : Displays this help
* -s | --strip            : Strips the current folder files and create a file with rustdoc information (comments.cmts by default)
* -g | --regenerate       : Recreate files with rustdoc comments from reading rustdoc information file (comments.cmts by default)
* -n | --no-file-output   : Display rustdoc information directly on stdout
* -i | --ignore [filename]: Ignore the specified file, can be repeated as much as needed, only used when stripping files, ignored otherwise
* -d | --dir [directory]  : Specify a directory path to work on, optional
* -v | --verbose          : Activate verbose mode
* -f | --force            : Remove confirmation demands
* -m | --ignore-macros    : macros in hierarchy will be ignored (so only macros with doc comments will appear in the comments file)
* -o | --comment-file     : specify the file within you want to save rustdoc information

By default, rustdoc is run with -s option:

```Shell
./rustdoc-stripper -s
```

IMPORTANT: Only files ending with '.rs' will be stripped/regenerated.
