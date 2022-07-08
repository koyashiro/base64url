use std::{
    fs::File,
    io::{stdin, stdout, Read, Write},
    path::PathBuf,
    str::FromStr,
};

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
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "-" => Ok(Self::Stdin),
            _ => Ok(Self::PathBuf(s.try_into()?)),
        }
    }
}

fn encode(mut input: impl Read, mut output: impl Write) -> anyhow::Result<()> {
    let decoded = {
        let mut buf = Vec::new();
        input.read_to_end(&mut buf)?;
        buf
    };
    let encoded = base64_url::encode(&decoded);

    writeln!(output, "{encoded}")?;

    Ok(())
}

fn decode(mut input: impl Read, mut output: impl Write) -> anyhow::Result<()> {
    let mut buf = String::new();
    input.read_to_string(&mut buf)?;
    let encoded = buf.trim_end();
    let decoded = base64_url::decode(&encoded)?;

    output.write_all(&decoded)?;

    Ok(())
}

fn execute(stdin: impl Read, stdout: impl Write, args: Args) -> anyhow::Result<()> {
    let input = match args.file.into() {
        FileKind::PathBuf(p) => Box::new(File::open(p)?) as Box<dyn Read>,
        FileKind::Stdin => Box::new(stdin) as Box<dyn Read>,
    };

    if args.decode {
        decode(input, stdout)?;
    } else {
        encode(input, stdout)?;
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let stdin = stdin();
    let stdout = stdout();
    let args = Args::parse();

    execute(stdin, stdout, args)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use tempfile::NamedTempFile;

    use super::*;

    const CASES: [(&[u8], &[u8]); 4] = [
        (b"hello", b"aGVsbG8"),
        (b"John Doe", b"Sm9obiBEb2U"),
        (b"\xF0\x9F\x8D\xA3", b"8J-Now"),
        (
            b"\xde\x9a\x4c\x32\x9e\x0d\x5b\xa8\x39\xed\x33\x5b\xe1\x9c\x01\xd9",
            b"3ppMMp4NW6g57TNb4ZwB2Q",
        ),
    ];

    #[test]
    fn encode_test() {
        for (raw, encoded) in CASES {
            let mut input = Cursor::new(raw);
            let mut output = Vec::new();
            encode(&mut input, &mut output).unwrap();
            assert_eq!(output, encoded);
        }
    }

    #[test]
    fn decode_test() {
        for (raw, encoded) in CASES {
            let mut input = Cursor::new(encoded);
            let mut output = Vec::new();
            decode(&mut input, &mut output).unwrap();
            assert_eq!(output, raw);
        }
    }

    #[test]
    fn execute_encode_none_input_test() {
        for (raw, encoded) in CASES {
            let mut stdin = Cursor::new(raw);
            let mut stdout = Vec::new();
            let args = Args {
                decode: false,
                file: None,
            };
            execute(&mut stdin, &mut stdout, args).unwrap();
            assert_eq!(stdout, encoded);
        }
    }

    #[test]
    fn execute_encode_stdin_input_test() {
        for (raw, encoded) in CASES {
            let mut stdin = Cursor::new(raw);
            let mut stdout = Vec::new();
            let args = Args {
                decode: false,
                file: Some(FileKind::Stdin),
            };
            execute(&mut stdin, &mut stdout, args).unwrap();
            assert_eq!(stdout, encoded);
        }
    }

    #[test]
    fn execute_encode_file_input_test() {
        for (raw, encoded) in CASES {
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
            execute(&mut stdin, &mut stdout, args).unwrap();
            assert_eq!(stdout, encoded);
        }
    }

    #[test]
    fn execute_decode_none_input_test() {
        for (raw, encoded) in CASES {
            let mut stdin = Cursor::new(encoded);
            let mut stdout = Vec::new();
            let args = Args {
                decode: true,
                file: None,
            };
            execute(&mut stdin, &mut stdout, args).unwrap();
            assert_eq!(stdout, raw);
        }
    }

    #[test]
    fn execute_decode_stdin_input_test() {
        for (raw, encoded) in CASES {
            let mut stdin = Cursor::new(encoded);
            let mut stdout = Vec::new();
            let args = Args {
                decode: true,
                file: Some(FileKind::Stdin),
            };
            execute(&mut stdin, &mut stdout, args).unwrap();
            assert_eq!(stdout, raw);
        }
    }

    #[test]
    fn execute_decode_file_input_test() {
        for (raw, encoded) in CASES {
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
            execute(&mut stdin, &mut stdout, args).unwrap();
            assert_eq!(stdout, raw);
        }
    }
}
