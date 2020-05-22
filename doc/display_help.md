# Displaying Help

You can run SpecDown with no sub-commands and it will display the help.

```shell,script(name="with-no-args")
target/debug/specdown 2>&1
```

Outputs:

```,verify(script_name="with-no-args", stream=output)
specdown 
A tool to test markdown files and drive devlopment from documentation.

USAGE:
    specdown [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    help    Prints this message or the help of the given subcommand(s)
    run     Runs a given Markdown Specification.
```

## Sub-commands

You can also run a specific sub-command with the `--help` argument for help on that sub-command.
For example:

```shell,script(name="run-with-help")
target/debug/specdown run --help 2>&1
```

Displays:

```,verify(script_name="run-with-help", stream=output)
specdown-run 
Runs a given Markdown Specification.

USAGE:
    specdown run <spec-file>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <spec-file>    The spec file to run
```