use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader, Error as IoError};
use std::num::ParseIntError;
use std::path::Path;
use std::str::FromStr;

#[derive(Debug)]
enum Error {
    Io(IoError),
    ParseInt(ParseIntError),
    Invalid,
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

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Io(e) => fmt::Display::fmt(e, f),
            Error::ParseInt(e) => fmt::Display::fmt(e, f),
            Error::Invalid => write!(f, "invalid input"),
        }
    }
}

#[derive(Debug)]
struct Coordinate {
    x: i8,
    y: i8,
    z: i8,
    t: i8,
}

impl Coordinate {
    fn distance(&self, other: &Self) -> i8 {
        (self.x - other.x).abs()
            + (self.y - other.y).abs()
            + (self.z - other.z).abs()
            + (self.t - other.t).abs()
    }
}

impl FromStr for Coordinate {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let mut split = string.split(',');
        let (x, y, z, t) = match (split.next(), split.next(), split.next(), split.next()) {
            (Some(x), Some(y), Some(z), Some(t)) => {
                (x.parse()?, y.parse()?, z.parse()?, t.parse()?)
            }
            _ => return Err(Error::Invalid),
        };

        Ok(Coordinate { x, y, z, t })
    }
}

fn parse_input(path: &Path) -> Result<Vec<Coordinate>, Error> {
    let coordinates = BufReader::new(File::open(path)?)
        .lines()
        .map(|line| line.map_err(Error::from).and_then(|s| s.parse()))
        .collect::<Result<Vec<Coordinate>, Error>>()?;

    if !coordinates.is_empty() {
        Ok(coordinates)
    } else {
        Err(Error::Invalid)
    }
}

fn explore(vec: &[bool], len: usize, index: usize, visited: &mut [bool]) {
    for i in vec
        .chunks_exact(len)
        .nth(index)
        .unwrap()
        .iter()
        .enumerate()
        .filter_map(|(i, &close)| if close { Some(i) } else { None })
    {
        if !visited[i] {
            visited[i] = true;
            explore(vec, len, i, visited);
        }
    }
}

fn constellations(coordinates: &[Coordinate]) -> i32 {
    let len = coordinates.len();
    let mut vec = vec![false; len * len];
    for (i, coordinate) in coordinates.iter().enumerate() {
        for (j, other) in coordinates.iter().enumerate().skip(i + 1) {
            if coordinate.distance(other) < 4 {
                vec[i * len + j] = true;
                vec[j * len + i] = true;
            }
        }
    }

    let mut counter = 0;
    let mut visited = vec![false; len];

    for i in 0..len {
        if visited[i] {
            continue;
        } else {
            visited[i] = true;
            counter += 1;
            explore(&vec, len, i, &mut visited);
        }
    }

    counter
}

fn main() -> Result<(), Error> {
    let path = Path::new("inputs/input-25-01.txt");

    let coordinates = parse_input(path)?;

    println!("Part 1: {}", constellations(&coordinates));

    Ok(())
}
