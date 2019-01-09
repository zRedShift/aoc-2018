use std::fmt;
use std::fs::File;
use std::io::{prelude::*, BufReader, Error as IoError};
use std::path::Path;

const TABLE: usize = 1 << 5;
const MASK: usize = 0b0001_1111;
const GEN_1: usize = 20;
const GEN_2: usize = 50_000_000_000;

#[derive(Debug)]
enum Error {
    Io(IoError),
    Invalid,
}

impl From<IoError> for Error {
    fn from(error: IoError) -> Self {
        Error::Io(error)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Io(e) => fmt::Display::fmt(e, f),
            Error::Invalid => write!(f, "invalid input"),
        }
    }
}

fn parse_input(path: &Path) -> Result<(Vec<u8>, [u8; TABLE]), Error> {
    let mut buf = BufReader::new(File::open(path)?).lines();

    let s = match buf.next() {
        Some(Ok(s)) => {
            if s.len() > 17 {
                s
            } else {
                return Err(Error::Invalid);
            }
        }
        Some(Err(e)) => return Err(e.into()),
        None => return Err(Error::Invalid),
    };

    let s = &s[15..];

    let mut initial = Vec::with_capacity(s.len());

    for b in s.bytes() {
        match b {
            b'.' => initial.push(0),
            b'#' => initial.push(1),
            _ => return Err(Error::Invalid),
        }
    }

    let mut table = [0u8; TABLE];

    for s in buf.skip(1).take(TABLE) {
        let s = s?;

        match s.bytes().nth(9) {
            Some(b'.') => continue,
            Some(b'#') => (),
            _ => return Err(Error::Invalid),
        }

        let mut index = 0;

        for b in s.bytes().take(5) {
            index <<= 1;

            match b {
                b'.' => (),
                b'#' => index |= 1,
                _ => return Err(Error::Invalid),
            }
        }
        table[index] = 1;
    }

    Ok((initial, table))
}

fn advance(prev: &[u8], table: &[u8; TABLE]) -> (Vec<u8>, i64) {
    let mut next = Vec::with_capacity(prev.len() + 4);
    let mut index = 0;
    let mut first = false;
    let mut shift = -2;

    for &x in prev.iter().chain([0, 0, 0, 0].iter()) {
        index = index << 1 | x as usize;

        match (first, table[index & MASK]) {
            (false, 1) => {
                first = true;
                next.push(1);
            }
            (true, x) => next.push(x),
            _ => shift += 1,
        }
    }

    while next.last() == Some(&0) {
        next.pop();
    }

    (next, shift)
}

fn evolve(initial: &[u8], table: &[u8; TABLE], generations: usize) -> i64 {
    let mut shift = 0;
    let mut vec = Vec::from(initial);

    for i in 0..generations {
        let (v, s) = advance(&vec, table);

        if vec == v {
            shift += (generations - i) as i64 * s;
            break;
        }

        shift += s;

        vec = v;
    }

    vec.iter()
        .enumerate()
        .filter_map(|(i, &x)| if x == 1 { Some(i as i64 + shift) } else { None })
        .sum()
}

fn main() -> Result<(), Error> {
    let path = Path::new("inputs/input-12-01.txt");

    let (initial, table) = parse_input(path)?;

    println!("Part 1: {:?}", evolve(&initial, &table, GEN_1));
    println!("Part 2: {:?}", evolve(&initial, &table, GEN_2));

    Ok(())
}
