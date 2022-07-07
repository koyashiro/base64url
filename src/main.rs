use std::{
    fs::File,
    io::{stdin, stdout, Read, Write},
    path::PathBuf,
    str::FromStr,
};

use clap::Parser;

const STDIN: &str = "-";

#[derive(Debug, Parser)]
#[clap(about, author, version)]
struct Args {
    #[clap(long, short)]
    decode: bool,

    #[clap(default_value = STDIN, value_parser)]
    file: FileKind,
}

#[derive(Clone, Debug)]
enum FileKind {
    PathBuf(PathBuf),
    Stdin,
}

impl FromStr for FileKind {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            STDIN => Ok(Self::Stdin),
            _ => Ok(Self::PathBuf(s.try_into()?)),
        }
    }
}

fn encode(mut input: impl Read, mut output: impl Write) -> anyhow::Result<()> {
    let input = {
        let mut buf = Vec::new();
        input.read_to_end(&mut buf)?;
        buf
    };
    let encoded = base64_url::encode(&input);
    output.write_all(encoded.as_bytes())?;

    Ok(())
}

fn decode(mut input: impl Read, mut output: impl Write) -> anyhow::Result<()> {
    let input = {
        let mut buf = Vec::new();
        input.read_to_end(&mut buf)?;
        buf
    };
    let decoded = base64_url::decode(&input)?;
    output.write_all(&decoded)?;

    Ok(())
}

fn execute(stdin: impl Read, stdout: impl Write, args: Args) -> anyhow::Result<()> {
    let input = match args.file {
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

    use super::*;

    #[test]
    fn encode_test() {
        let input = Cursor::new(b"hello");
        let mut output = Vec::new();
        encode(input, &mut output).unwrap();
        assert_eq!(output, b"aGVsbG8");
    }

    #[test]
    fn decode_test() {
        let input = Cursor::new(b"aGVsbG8");
        let mut output = Vec::new();
        decode(input, &mut output).unwrap();
        assert_eq!(output, b"hello");
    }

    #[test]
    fn execute_stdin_encode_test() {
        let stdin = Cursor::new(b"hello");
        let mut stdout = Vec::new();
        let args = Args {
            decode: false,
            file: FileKind::Stdin,
        };
        execute(stdin, &mut stdout, args).unwrap();
        assert_eq!(stdout, b"aGVsbG8");
    }

    #[test]
    fn execute_stdin_decode_test() {
        let stdin = Cursor::new(b"aGVsbG8");
        let mut stdout = Vec::new();
        let args = Args {
            decode: true,
            file: FileKind::Stdin,
        };
        execute(stdin, &mut stdout, args).unwrap();
        assert_eq!(stdout, b"hello");
    }
}
