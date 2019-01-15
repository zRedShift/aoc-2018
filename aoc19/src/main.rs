use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader, Error as IoError};
use std::num::ParseIntError;
use std::path::Path;
use std::str::FromStr;

const INSTRUCTION_COUNT: usize = 16;
const REGISTER_SIZE: usize = 6;

type Registers = [usize; REGISTER_SIZE];
type OpCode = fn(&mut Registers, usize, usize, usize);
type InstructionSet = [OpCode; INSTRUCTION_COUNT];

const INSTRUCTIONS: InstructionSet = [
    |reg, a, b, c| reg[c] = reg[a] + reg[b],
    |reg, a, b, c| reg[c] = reg[a] + b,
    |reg, a, b, c| reg[c] = reg[a] * reg[b],
    |reg, a, b, c| reg[c] = reg[a] * b,
    |reg, a, b, c| reg[c] = reg[a] & reg[b],
    |reg, a, b, c| reg[c] = reg[a] & b,
    |reg, a, b, c| reg[c] = reg[a] | reg[b],
    |reg, a, b, c| reg[c] = reg[a] | b,
    |reg, a, _, c| reg[c] = reg[a],
    |reg, a, _, c| reg[c] = a,
    |reg, a, b, c| reg[c] = if a > reg[b] { 1 } else { 0 },
    |reg, a, b, c| reg[c] = if b < reg[a] { 1 } else { 0 },
    |reg, a, b, c| reg[c] = if reg[a] > reg[b] { 1 } else { 0 },
    |reg, a, b, c| reg[c] = if a == reg[b] { 1 } else { 0 },
    |reg, a, b, c| reg[c] = if b == reg[a] { 1 } else { 0 },
    |reg, a, b, c| reg[c] = if reg[a] == reg[b] { 1 } else { 0 },
];

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

impl From<String> for Error {
    fn from(error: String) -> Self {
        Error::Invalid(error)
    }
}

fn factor_sum(mut num: usize) -> usize {
    let sqrt = (num as f64).sqrt() as usize + 1;
    let mut res = 1;

    for i in 2..sqrt {
        let mut sum = 1;
        let mut term = 1;

        while num % i == 0 {
            num /= i;
            term *= i;
            sum += term;
        }

        res *= sum
    }

    if num > 2 {
        res *= 1 + num;
    }

    res
}

#[derive(Debug)]
struct Device {
    registers: Registers,
    instruction_pointer: usize,
}

impl Device {
    fn reset(&mut self) {
        self.registers[0] = 1;
        for r in self.registers[1..].iter_mut() {
            *r = 0;
        }
    }

    fn execute(&mut self, instructions: &[Instruction]) -> usize {
        while let Some(instruction) = instructions.get(self.registers[self.instruction_pointer]) {
            if instruction.opcode == 15 {
                return factor_sum(if instruction.a == instruction.b {
                    self.registers[instruction.c]
                } else if instruction.a == instruction.c {
                    self.registers[instruction.b]
                } else {
                    self.registers[instruction.a]
                });
            }

            INSTRUCTIONS[instruction.opcode](
                &mut self.registers,
                instruction.a,
                instruction.b,
                instruction.c,
            );

            self.registers[self.instruction_pointer] += 1;
        }

        self.registers[0]
    }
}

impl FromStr for Device {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < 5 || &s[0..4] != "#ip " {
            Err(Error::from("missing instruction pointer"))
        } else {
            s[4..]
                .parse()
                .map_err(Error::from)
                .and_then(|instruction_pointer| {
                    if instruction_pointer < 6 {
                        Ok(Device {
                            registers: [0; REGISTER_SIZE],
                            instruction_pointer,
                        })
                    } else {
                        Err(Error::from("instruction pointer out of bounds"))
                    }
                })
        }
    }
}

#[derive(Debug)]
struct Instruction {
    opcode: usize,
    a: usize,
    b: usize,
    c: usize,
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

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Io(e) => fmt::Display::fmt(e, f),
            Error::ParseInt(e) => fmt::Display::fmt(e, f),
            Error::Invalid(s) => write!(f, "{}", s),
        }
    }
}

fn parse_input(path: &Path) -> Result<(Device, Vec<Instruction>), Error> {
    let mut lines = BufReader::new(File::open(path)?).lines();

    let device = lines
        .next()
        .ok_or_else(|| Error::from("unexpected EOF"))??
        .parse()?;

    let instructions = lines
        .map(|result| result.map_err(Error::from).and_then(|s| s.parse()))
        .collect::<Result<_, _>>()?;

    Ok((device, instructions))
}

fn main() -> Result<(), Error> {
    let path = Path::new("inputs/input-19-01.txt");

    let (mut device, instructions) = parse_input(path)?;

    println!("Part 1: {}", device.execute(&instructions));

    device.reset();

    println!("Part 2: {}", device.execute(&instructions));

    Ok(())
}
