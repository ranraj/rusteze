# rusteze
Command line application that manage server entires and connect through ssh. 

Supporting Operating System :  MAC

```
cargo build
```
cd to target bin
```
❯ ./rusteze
Todo 1.0
Ranjith Raj D <ranjithraj.d@gmail.com>
Todo app

USAGE:
    rusteze [FLAGS] [OPTIONS] [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -s, --set        Set currently passed flags are default
    -V, --version    Prints version information

OPTIONS:
    -d, --db <database name>    Sets a custom database name

SUBCOMMANDS:
    add        Insert todo into the application
    connect    Connect to ssh server for the given input
    help       Prints this message or the help of the given subcommand(s)
    init       Initialize the db for first time setup
    list       Lists all todo without argument otherwise give argument
    remove     Remove all todo without argument otherwise give argument
    test       Test the application status

```
