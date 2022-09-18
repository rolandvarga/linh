# LINH
A cli command to track other cli commands.

### Overview
The tool is best used as a cli command that's listed in `$PATH` for easy access. The executable stores entries in a file called `entries.json` located at `$HOME/.cargo/entries.json`. It currently supports adding new entries, listing them & filtering based on a substring.

### Install
The tool can be used directly via cargo or installed as a binary

#### via cargo
```bash
> cd linh
> cargo run -- list
    Finished dev [unoptimized + debuginfo] target(s) in 0.04s
     Running `target/debug/linh list`
 INFO  linh >
+----+---------+------------------------+
| ID | COMMAND | DESCRIPTION            |
+=======================================+
| 1  | ls -la  | listing all the things |
|----+---------+------------------------|
| 2  | ls -lR  | listing recursively    |
+----+---------+------------------------+
```

#### via binary
```bash
> cd linh
> cargo build --release
> cargo install --path .
> export PATH=$PATH:$HOME/.cargo/bin;
> cd $HOME
> linh --help
lin-help 0.1.0
a handy tool for collecting common shell commands

USAGE:
    linh [SUBCOMMAND]

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    add       adds a new command to the list
    help      Print this message or the help of the given subcommand(s)
    list      list all available commands
    search    shows all available commands for the search term
```

### Usage
The following are examples of the available commands.

The tool comes with a `--help` option, and subcommands also have their help messages:
```bash
> linh --help

USAGE:
    linh [SUBCOMMAND]

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    add       adds a new command to the list
    help      Print this message or the help of the given subcommand(s)
    list      list all available commands
    search    shows all available commands for the search term

> linh help add
linh-add
adds a new command to the list

USAGE:
    linh add <COMMAND> <DESCRIPTION>

ARGS:
    <COMMAND>        command to save, should be in quotes
    <DESCRIPTION>    short description of the provided command

OPTIONS:
    -h, --help    Print help information
```

Arguments should be surrounded by quotes, when adding or searching for entries:

```bash
# adding a command
> linh add 'mkdir -p' 'create directory with intermediate directories as required'
INFO  linh::model > successfully saved entry

# searching for entries
> linh search 'ls'
+----+---------+------------------------+
| ID | COMMAND | DESCRIPTION            |
+=======================================+
| 2  | ls -la  | listing all the things |
|----+---------+------------------------|
| 3  | ls -lR  | listing recursively    |
+----+---------+------------------------+
```