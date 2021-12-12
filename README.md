# rustic-toolz 🦀 🛠  🔐 ⚡️ ✨

[![CI](https://github.com/gabrielfalcao/rustic-toolz/actions/workflows/rust.yml/badge.svg)](https://github.com/gabrielfalcao/rustic-toolz/actions/workflows/rust.yml)


## Developing

### "Unit" Testing

```bash
cargo test
```

### "End-to-end" Testing

```bash
make test
```


### Building

```bash
cargo build --release
```


## Installing

```
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


- Performs aes-256-cbc encryption and decryption.
- PBKDF2 HMAC 256 for password-based key derivation with configurable number of cycles.



#### Example YAML configuration

**~/.rustic-toolz.yaml**

Configure PBKDF2 iteration count for each key material type to custom numbers.
Higher numbers means that keys will take longer to generate and are safer.

```yaml
cycles:
  key: 500
  salt: 300
  iv: 1200
```
> NOTE: Keys created with a different combination of cycles cannot be derived again.

#### Generating a key file based on password

This step is optional if you want to provide a password in every encryption/decryption process.

**`-k` or `--key-filename`**
> Path to the key file where the key file will be stored.

**`-p` or `--ask-password`**
> Input the password safely with confirmation

**`-P` or `--password`** `<password>`
> Input the password as command-line argument.

##### Example

```bash
aes-256-cbc generate \
    --key-filename ~/.personal-aes-key.yml \
    --ask-password
```

[![asciicast](https://asciinema.org/a/ogEf12HY2ngDb0CzoelLhlOBt.svg)](https://asciinema.org/a/ogEf12HY2ngDb0CzoelLhlOBt)


#### `aes-256-cbc encrypt`


**`-k` or `--key-filename`**
> Path to the key file where the key file used for encryption. Required unless `--ask-password` is used.

**`-p` or `--ask-password`**
> Input the encryption password safely with confirmation. Required unless `--password` is used.

**`-P` or `--password`** `<password>`
> Input the encryption password as command-line argument. Required unless `--key-filename` is used.

**`-i` or `--input-filename`** `<filename>`
> The plaintext file

**`-o` or `--output-filename`** `<filename>`
> The file where the encrypted (cyphertext) will be stored. (pass the same as the input filename to replace the file)


##### Example: Encrypting file using password

```bash
aes-256-cbc encrypt \
    --password 'I <3 Nickelback XOXO' \
    --input-filename=Cargo.toml --output-filename=Cargo.toml.aes
```

[![asciicast](https://asciinema.org/a/lIZWbm1SIvNjdl5ZI0DeYlo4T.svg)](https://asciinema.org/a/lIZWbm1SIvNjdl5ZI0DeYlo4T)


##### Example: Decrypting file using password

```bash
aes-256-cbc decrypt \
    --password 'I <3 Nickelback XOXO'  \
    --input-filename=Cargo.toml.aes --output-filename=Cargo.toml
```

[![asciicast](https://asciinema.org/a/zr6wKh4psf25bYhzlqDcJfppe.svg)](https://asciinema.org/a/zr6wKh4psf25bYhzlqDcJfppe)

##### Example: encrypt file using pre-generated key

```bash
aes-256-cbc encrypt \
    --key-filename ~/.personal-aes-key.yml \
    --input-filename=Cargo.toml --output-filename=Cargo.toml.aes
```

[![asciicast](https://asciinema.org/a/rCPLPZrGHwUQYTbFR8tDZPCRn.svg)](https://asciinema.org/a/rCPLPZrGHwUQYTbFR8tDZPCRn)


##### Example: Decrypting file using a key file


**`-k` or `--key-filename`**
> Path to the key file where the key file used for decryption. Required unless `--ask-password` is used.

**`-p` or `--ask-password`**
> Input the decryption password safely with confirmation. Required unless `--password` is used.

**`-P` or `--password`** `<password>`
> Input the decryption password as command-line argument. Required unless `--key-filename` is used.

**`-i` or `--input-filename`** `<filename>`
> The encrypted (cyphertext) file to be decrypted

**`-o` or `--output-filename`** `<filename>`
> The file where the decrypted (plaintext) will be stored. (pass the same as the input filename to replace the file)


```bash
aes-256-cbc decrypt \
    --key-filename ~/.personal-aes-key.yml \
    --input-filename=Cargo.toml.aes --output-filename=Cargo.toml
```

[![asciicast](https://asciinema.org/a/Wp4Q5PTDbFHDptYiYW9dRwGxd.svg)](https://asciinema.org/a/Wp4Q5PTDbFHDptYiYW9dRwGxd)
