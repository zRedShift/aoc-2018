use std::collections::HashSet;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader, Error as IoError};
use std::num::ParseIntError;
use std::path::Path;
use std::str::FromStr;

const SMALL_AND: i32 = (1 << 8) - 1;
const BIG_AND: i32 = (1 << 24) - 1;
const SHIFT: i32 = 8;
const OR: i32 = 1 << 16;

#[derive(Debug)]
enum Error {
    Io(IoError),
    ParseInt(ParseIntError),
    Invalid(String),
}

impl From<IoError> for Error {
    fn from(error: IoError) -> Self {
        Error::Io(error)
    }
}

impl From<ParseIntError> for Error {
    fn from(error: ParseIntError) -> Self {
        Error::ParseInt(error)
    }
}

impl From<&str> for Error {
    fn from(error: &str) -> Self {
        Error::Invalid(error.into())
    }
}


impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Io(e) => fmt::Display::fmt(e, f),
            Error::ParseInt(e) => fmt::Display::fmt(e, f),
            Error::Invalid(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug)]
struct Device;

impl FromStr for Device {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < 5 || &s[0..4] != "#ip " {
            Err(Error::from("missing instruction pointer"))
        } else {
            s[4..]
                .parse::<u8>()
                .map_err(Error::from)
                .and_then(|instruction_pointer| {
                    if instruction_pointer < 6 {
                        Ok(Device)
                    } else {
                        Err(Error::from("instruction pointer out of bounds"))
                    }
                })
        }
    }
}

#[derive(Debug)]
struct Instruction {
    opcode: i32,
    a: i32,
    b: i32,
    c: i32,
}

impl FromStr for Instruction {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split_whitespace();

        let (opcode, a, b, c) = match (split.next(), split.next(), split.next(), split.next()) {
            (Some(opcode), Some(a), Some(b), Some(c)) => {
                (opcode, a.parse()?, b.parse()?, c.parse()?)
            }
            _ => return Err(Error::from("invalid instruction syntax")),
        };

        let opcode = match opcode {
            "addr" => 0,
            "addi" => 1,
            "mulr" => 2,
            "muli" => 3,
            "banr" => 4,
            "bani" => 5,
            "borr" => 6,
            "bori" => 7,
            "setr" => 8,
            "seti" => 9,
            "gtir" => 10,
            "gtri" => 11,
            "gtrr" => 12,
            "eqir" => 13,
            "eqri" => 14,
            "eqrr" => 15,
            opcode => return Err(Error::Invalid(format!("invalid opcode {}", opcode))),
        };

        Ok(Instruction { opcode, a, b, c })
    }
}

fn parse_input(path: &Path) -> Result<(i32, i32), Error> {
    let mut lines = BufReader::new(File::open(path)?).lines();

    let _ = lines
        .next()
        .ok_or_else(|| Error::from("unexpected EOF"))??
        .parse::<Device>()?;

    let instructions = lines
        .map(|result| result.map_err(Error::from).and_then(|s| s.parse()))
        .collect::<Result<Vec<Instruction>, _>>()?;

    Ok((instructions[7].a, instructions[11].b))
}

fn compute(mut ans: i32, mult: i32, mut add: i32) -> i32 {
    ans += add & SMALL_AND;
    ans *= mult;
    ans &= BIG_AND;
    add >>= SHIFT;
    ans += add & SMALL_AND;
    ans *= mult;
    ans &= BIG_AND;
    add >>= SHIFT;
    ans += add & SMALL_AND;
    ans *= mult;
    ans & BIG_AND
}

fn fast_execute(seed: i32, mult: i32) -> (i32, i32) {
    let mut add = OR;
    let mut add_set = HashSet::new();
    let mut ans_set = HashSet::new();

    add_set.insert(add);
    let first = compute(seed, mult, add);
    let mut last = first;
    add = first | OR;

    while add_set.insert(add) {
        let ans = compute(seed, mult, add);
        add = ans | OR;
        if ans_set.insert(ans) {
            last = ans;
        }
    }

    (first, last)
}

fn main() -> Result<(), Error> {
    let path = Path::new("inputs/input-21-01.txt");

    let (seed, mult) = parse_input(path)?;

    let (first, last) = fast_execute(seed, mult);

    println!("Part 1: {}", first);
    println!("Part 2: {}", last);

    Ok(())
}
