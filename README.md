# base64url

Base64url encode or decode FILE or standard input, to standard output.

## Install

```sh
cargo install --git https://github.com/koyashiro/base64url
```

## Usage

```console
$ base64url --help
Base64url encode or decode FILE or standard input, to standard output.

USAGE:
    base64url [OPTIONS] [FILE]

ARGS:
    <FILE>    With no FILE, or when FILE is -, read standard input

OPTIONS:
    -d, --decode     Decode data
    -h, --help       Print help information
    -V, --version    Print version information
```

### Encode

Encode string.

```console
$ echo -n hello | base64url
aGVsbG8
```

Encode binary.

```console
$ head -c 16 /dev/random | base64url
6tp3BcfXk8-ku2eeSH6-7w
```

### Decode

Decode to string.

```console
$ echo -n aGVsbG8 | base64url -d
hello
```

Decode to binary.

```console
$ echo -n 6tp3BcfXk8-ku2eeSH6-7w | base64url -d | hexdump
0000000 daea 0577 d7c7 cf93 bba4 9e67 7e48 efbe
0000010
```
