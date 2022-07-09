use std::{
    convert::Infallible,
    fs::File,
    io::{stdin, stdout, Read, Write},
    path::PathBuf,
    str::FromStr,
};

use base64::{decode_config, encode_config, URL_SAFE_NO_PAD};
use clap::Parser;

#[derive(Debug, Parser)]
#[clap(about, version)]
struct Args {
    /// Decode data.
    #[clap(long, short)]
    decode: bool,

    /// With no FILE, or when FILE is -, read standard input.
    #[clap(value_parser)]
    file: Option<FileKind>,
}

#[derive(Clone, Debug)]
enum FileKind {
    PathBuf(PathBuf),
    Stdin,
}

impl From<Option<FileKind>> for FileKind {
    fn from(file_kind: Option<FileKind>) -> Self {
        match file_kind {
            Some(fk) => fk,
            None => Self::Stdin,
        }
    }
}

impl FromStr for FileKind {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "-" => Ok(Self::Stdin),
            _ => Ok(Self::PathBuf(s.into())),
        }
    }
}

fn encode(mut input: impl Read, mut output: impl Write) -> Result<(), anyhow::Error> {
    let decoded = {
        let mut buf = Vec::new();
        input.read_to_end(&mut buf)?;
        buf
    };
    let encoded = encode_config(&decoded, URL_SAFE_NO_PAD);

    writeln!(output, "{encoded}")?;

    Ok(())
}

fn decode(mut input: impl Read, mut output: impl Write) -> Result<(), anyhow::Error> {
    let mut buf = String::new();
    input.read_to_string(&mut buf)?;
    let encoded = buf.trim_end();
    let decoded = decode_config(&encoded, URL_SAFE_NO_PAD)?;

    output.write_all(&decoded)?;

    Ok(())
}

fn execute(stdin: impl Read, stdout: impl Write, args: &Args) -> Result<(), anyhow::Error> {
    match &args.file {
        Some(FileKind::PathBuf(p)) => {
            let file = File::open(p)?;
            if args.decode {
                decode(file, stdout)?;
            } else {
                encode(file, stdout)?;
            }
        }
        None | Some(FileKind::Stdin) => {
            if args.decode {
                decode(stdin, stdout)?;
            } else {
                encode(stdin, stdout)?;
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), anyhow::Error> {
    let mut stdin = stdin();
    let mut stdout = stdout();
    let args = Args::parse();

    execute(&mut stdin, &mut stdout, &args)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use tempfile::NamedTempFile;

    use super::*;

    const ENCODE_TEST_CASES: [(&[u8], &[u8]); 8] = [
        (b"hello", b"aGVsbG8"),
        (b"hello\n", b"aGVsbG8K"),
        (b"John Doe", b"Sm9obiBEb2U"),
        (b"John Doe\n", b"Sm9obiBEb2UK"),
        (b"\xF0\x9F\x8D\xA3", b"8J-Now"),
        (b"\xF0\x9F\x8D\xA3\n", b"8J-Nowo"),
        (
            b"\xde\x9a\x4c\x32\x9e\x0d\x5b\xa8\x39\xed\x33\x5b\xe1\x9c\x01\xd9",
            b"3ppMMp4NW6g57TNb4ZwB2Q",
        ),
        (
            b"\xde\x9a\x4c\x32\x9e\x0d\x5b\xa8\x39\xed\x33\x5b\xe1\x9c\x01\xd9\n",
            b"3ppMMp4NW6g57TNb4ZwB2Qo",
        ),
    ];

    const TRAILING_WHITESPACES: [&[u8]; 7] = [b"", b" ", b"  ", b"   ", b"\n", b"\n\n", b"\n\n\n"];

    #[cfg(test)]
    mod encode {
        use super::*;

        #[test]
        fn it_writes_encoded_bytes() {
            for (raw, encoded) in ENCODE_TEST_CASES {
                let mut input = Cursor::new(raw);
                let mut output = Vec::new();
                assert!(encode(&mut input, &mut output).is_ok());
                assert_eq!(output, [encoded, b"\n"].concat());
            }
        }
    }

    #[cfg(test)]
    mod decode {
        use super::*;

        #[test]
        fn it_writes_decoded_bytes() {
            for (raw, encoded) in ENCODE_TEST_CASES {
                let mut input = Cursor::new(encoded);
                let mut output = Vec::new();
                assert!(decode(&mut input, &mut output).is_ok());
                assert_eq!(output, raw);
            }
        }

        #[test]
        fn it_ignores_trailing_whitespace() {
            for trailing_whitespace in TRAILING_WHITESPACES {
                for (raw, encoded) in ENCODE_TEST_CASES {
                    let mut input = Cursor::new([encoded, trailing_whitespace].concat());
                    let mut output = Vec::new();
                    assert!(decode(&mut input, &mut output).is_ok());
                    assert_eq!(output, raw);
                }
            }
        }
    }

    #[cfg(test)]
    mod execute {
        use super::*;

        #[test]
        fn it_encodes_standard_input() {
            let argss = [
                Args {
                    decode: false,
                    file: None,
                },
                Args {
                    decode: false,
                    file: Some(FileKind::Stdin),
                },
            ];
            for args in argss {
                for (raw, encoded) in ENCODE_TEST_CASES {
                    let mut stdin = Cursor::new(raw);
                    let mut stdout = Vec::new();
                    assert!(execute(&mut stdin, &mut stdout, &args).is_ok());
                    assert_eq!(stdout, [encoded, b"\n"].concat());
                }
            }
        }

        #[test]
        fn it_encode_file() {
            for (raw, encoded) in ENCODE_TEST_CASES {
                let mut stdin = Cursor::new(Vec::new());
                let mut stdout = Vec::new();
                let tempfile = {
                    let mut f = NamedTempFile::new().unwrap();
                    f.write_all(raw).unwrap();
                    f
                };
                let args = {
                    Args {
                        decode: false,
                        file: Some(FileKind::PathBuf(tempfile.path().into())),
                    }
                };
                assert!(execute(&mut stdin, &mut stdout, &args).is_ok());
                assert_eq!(stdout, [encoded, b"\n"].concat());
            }
        }

        #[test]
        fn it_decode_standard_input() {
            let argss = [
                Args {
                    decode: true,
                    file: None,
                },
                Args {
                    decode: true,
                    file: Some(FileKind::Stdin),
                },
            ];
            for args in argss {
                for (raw, encoded) in ENCODE_TEST_CASES {
                    let mut stdin = Cursor::new(encoded);
                    let mut stdout = Vec::new();
                    assert!(execute(&mut stdin, &mut stdout, &args).is_ok());
                    assert_eq!(stdout, raw);
                }
            }
        }

        #[test]
        fn it_decode_file() {
            for (raw, encoded) in ENCODE_TEST_CASES {
                let mut stdin = Cursor::new(Vec::new());
                let mut stdout = Vec::new();
                let tempfile = {
                    let mut f = NamedTempFile::new().unwrap();
                    f.write_all(encoded).unwrap();
                    f
                };
                let args = {
                    Args {
                        decode: true,
                        file: Some(FileKind::PathBuf(tempfile.path().into())),
                    }
                };
                assert!(execute(&mut stdin, &mut stdout, &args).is_ok());
                assert_eq!(stdout, raw);
            }
        }
    }
}
