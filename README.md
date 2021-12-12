# rustic-toolz 🦀 🛠  🔐 ⚡️ ✨

[![CI](https://github.com/gabrielfalcao/rustic-toolz/actions/workflows/rust.yml/badge.svg)](https://github.com/gabrielfalcao/rustic-toolz/actions/workflows/rust.yml)


## Building

```bash
cargo build --release
cp target/release/slugify-filenames /usr/local/bin/
cp target/release/aes-256-cbc /usr/local/bin/
```


## Command-line tools


### `slugify-filenames`

Slugifies a list of files or glob.

```man
USAGE:
    slugify-filenames [FLAGS] [target]...

FLAGS:
    -i, --case-insensitive    match glob case insensitive
    -n, --dry-run
    -h, --help                Prints help information
    -D, --include-hidden      include hidden files
    -r, --recursive           recurse directories
    -s, --silent
    -V, --version             Prints version information
    -v, --verbose

ARGS:
    <target>...    a glob pattern [default: *]
```

### `aes-256-cbc`

Symmetrical encryption

```manpage
USAGE:
    aes-256-cbc [FLAGS] [SUBCOMMAND]

FLAGS:
    -n, --dry-run
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    ask         ask for password and confirmation
    decrypt     decrypt file
    encrypt     encrypt file or string
    generate    generate key
    help        Prints this message or the help of the given subcommand(s)
```
