use std::collections::HashSet;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader, Error as IoError};
use std::iter;
use std::ops::{Index, IndexMut};
use std::path::Path;
use std::str::FromStr;

use lazy_static::lazy_static;
use regex::Regex;

const INSTRUCTION_COUNT: usize = 16;

type Instruction = fn(&mut Device, u32, u32, u32);
type InstructionSet = [Instruction; INSTRUCTION_COUNT];

const INSTRUCTIONS: InstructionSet = [
    |dev, a, b, c| dev[c] = dev[a] + dev[b],
    |dev, a, b, c| dev[c] = dev[a] + b,
    |dev, a, b, c| dev[c] = dev[a] * dev[b],
    |dev, a, b, c| dev[c] = dev[a] * b,
    |dev, a, b, c| dev[c] = dev[a] & dev[b],
    |dev, a, b, c| dev[c] = dev[a] & b,
    |dev, a, b, c| dev[c] = dev[a] | dev[b],
    |dev, a, b, c| dev[c] = dev[a] | b,
    |dev, a, _, c| dev[c] = dev[a],
    |dev, a, _, c| dev[c] = a,
    |dev, a, b, c| dev[c] = if a > dev[b] { 1 } else { 0 },
    |dev, a, b, c| dev[c] = if b < dev[a] { 1 } else { 0 },
    |dev, a, b, c| dev[c] = if dev[a] > dev[b] { 1 } else { 0 },
    |dev, a, b, c| dev[c] = if a == dev[b] { 1 } else { 0 },
    |dev, a, b, c| dev[c] = if b == dev[a] { 1 } else { 0 },
    |dev, a, b, c| dev[c] = if dev[a] == dev[b] { 1 } else { 0 },
];

#[derive(Debug)]
enum Error {
    Io(IoError),
    Invalid(String),
}

impl From<IoError> for Error {
    fn from(error: IoError) -> Self {
        Error::Io(error)
    }
}

impl From<&str> for Error {
    fn from(error: &str) -> Self {
        Error::Invalid(error.into())
    }
}

impl From<String> for Error {
    fn from(error: String) -> Self {
        Error::Invalid(error)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Io(e) => fmt::Display::fmt(e, f),
            Error::Invalid(s) => write!(f, "invalid input: {}", s),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Device([u32; 4]);

impl FromStr for Device {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex =
                Regex::new(r"(Before: |After:  )\[(\d), (\d), (\d), (\d)]").unwrap();
        }

        let caps = match RE.captures(s) {
            Some(caps) => caps,
            None => return Err("unrecognized device signature".into()),
        };

        Ok(Device([
            caps[2].parse().unwrap(),
            caps[3].parse().unwrap(),
            caps[4].parse().unwrap(),
            caps[5].parse().unwrap(),
        ]))
    }
}

impl Index<u32> for Device {
    type Output = u32;

    fn index(&self, idx: u32) -> &Self::Output {
        &self.0[idx as usize]
    }
}

impl IndexMut<u32> for Device {
    fn index_mut(&mut self, idx: u32) -> &mut Self::Output {
        &mut self.0[idx as usize]
    }
}

#[derive(Debug)]
struct Operation {
    opcode: usize,
    a: u32,
    b: u32,
    c: u32,
}

impl FromStr for Operation {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"(\d{1,2}) (\d) (\d) (\d)").unwrap();
        }

        let caps = match RE.captures(s) {
            Some(caps) => caps,
            None => return Err("unrecognized operation signature".into()),
        };

        Ok(Operation {
            opcode: caps[1].parse().unwrap(),
            a: caps[2].parse().unwrap(),
            b: caps[3].parse().unwrap(),
            c: caps[4].parse().unwrap(),
        })
    }
}

#[derive(Debug)]
struct Data {
    before: Device,
    after: Device,
    operation: Operation,
}

type Input = (Vec<Data>, Vec<Operation>);

fn parse_input(path: &Path) -> Result<Input, Error> {
    let mut lines = BufReader::new(File::open(path)?).lines();
    let mut data = Vec::new();

    loop {
        let before = match lines.next() {
            Some(s) => match s?.as_ref() {
                "" => break,
                s => s.parse()?,
            },
            None => return Err("unexpected EOF".into()),
        };

        let operation = match lines.next() {
            Some(s) => s?.parse()?,
            None => return Err("unexpected EOF".into()),
        };

        let after = match lines.next() {
            Some(s) => s?.parse()?,
            None => return Err("unexpected EOF".into()),
        };

        data.push(Data {
            before,
            after,
            operation,
        });

        lines.next();
    }

    lines.next();

    let operations: Result<Vec<_>, _> = lines
        .map(|result| result.map_err(Error::from).and_then(|s| s.parse()))
        .collect();

    Ok((data, operations?))
}

fn build_sets(data: &[Data]) -> (Vec<HashSet<usize>>, usize) {
    let mut sets = Vec::with_capacity(INSTRUCTION_COUNT);
    sets.extend(iter::repeat(HashSet::new()).take(INSTRUCTION_COUNT));

    let count = data
        .iter()
        .filter(|data| {
            INSTRUCTIONS
                .iter()
                .enumerate()
                .filter_map(|(i, &op)| {
                    let mut device = data.before.clone();

                    op(
                        &mut device,
                        data.operation.a,
                        data.operation.b,
                        data.operation.c,
                    );

                    if device == data.after {
                        sets[i].insert(data.operation.opcode);
                        Some(())
                    } else {
                        None
                    }
                })
                .count()
                > 2
        })
        .count();

    (sets, count)
}

fn map_instructions(sets: &mut [HashSet<usize>]) -> Result<InstructionSet, Error> {
    let mut transform = [0; 16];

    for _ in 0..INSTRUCTION_COUNT {
        let (i, opcode) = sets
            .iter_mut()
            .enumerate()
            .find(|(_, set)| set.len() == 1)
            .map(|(i, set)| (i, set.drain().next().unwrap()))
            .ok_or("unsolveable data")?;

        transform[opcode] = i;

        for set in sets.iter_mut() {
            set.remove(&opcode);
        }
    }

    Ok([
        INSTRUCTIONS[transform[0]],
        INSTRUCTIONS[transform[1]],
        INSTRUCTIONS[transform[2]],
        INSTRUCTIONS[transform[3]],
        INSTRUCTIONS[transform[4]],
        INSTRUCTIONS[transform[5]],
        INSTRUCTIONS[transform[6]],
        INSTRUCTIONS[transform[7]],
        INSTRUCTIONS[transform[8]],
        INSTRUCTIONS[transform[9]],
        INSTRUCTIONS[transform[10]],
        INSTRUCTIONS[transform[11]],
        INSTRUCTIONS[transform[12]],
        INSTRUCTIONS[transform[13]],
        INSTRUCTIONS[transform[14]],
        INSTRUCTIONS[transform[15]],
    ])
}

fn execute_procedure(instructions: InstructionSet, operations: &[Operation]) -> Device {
    let mut device = Device([0; 4]);

    for operation in operations {
        instructions[operation.opcode](&mut device, operation.a, operation.b, operation.c);
    }

    device
}

fn main() -> Result<(), Error> {
    let path = Path::new("inputs/input-16-01.txt");

    let (data, operations) = parse_input(path)?;

    let (mut sets, count) = build_sets(&data);

    println!("Part 1: {}", count);

    let instructions = map_instructions(&mut sets)?;

    let device = execute_procedure(instructions, &operations);

    println!("Part 2: {}", device[0]);

    Ok(())
}
