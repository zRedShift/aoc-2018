use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader, Error as IoError};
use std::mem;
use std::path::Path;

const DIM: usize = 50;
const PART_ONE: usize = 10;
const PART_TWO: usize = 1_000_000_000;
const THRESHOLD: usize = 500;
type Map = [[Object; DIM]; DIM];

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

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Io(e) => fmt::Display::fmt(e, f),
            Error::Invalid(s) => write!(f, "{}", s),
        }
    }
}

fn parse_input(path: &Path) -> Result<Map, Error> {
    let mut map = [[Object::OpenGround; DIM]; DIM];

    for (y, result) in BufReader::new(File::open(path)?).lines().enumerate() {
        let row = result?;

        for (x, object) in row.bytes().enumerate().take(DIM) {
            match object {
                b'|' => map[y][x] = Object::Trees,
                b'#' => map[y][x] = Object::Lumberyard,
                b'.' => (),
                b => {
                    return Err(Error::Invalid(format!(
                        "invalid character: {}",
                        char::from(b)
                    )))
                }
            }
        }
    }

    Ok(map)
}

#[derive(Copy, Clone)]
enum Object {
    OpenGround,
    Trees,
    Lumberyard,
}

fn change(map: &Map, x: usize, y: usize) -> Object {
    let mut trees = 0;
    let mut lumber = 0;

    let (x_min, x_max) = (
        if x == 0 { 0 } else { x - 1 },
        if x == DIM - 1 { DIM } else { x + 2 },
    );
    let (y_min, y_max) = (
        if y == 0 { 0 } else { y - 1 },
        if y == DIM - 1 { DIM } else { y + 2 },
    );

    for row in &map[y_min..y_max] {
        for &object in &row[x_min..x_max] {
            match object {
                Object::Trees => trees += 1,
                Object::Lumberyard => lumber += 1,
                Object::OpenGround => (),
            }
        }
    }

    match map[y][x] {
        Object::OpenGround if trees > 2 => Object::Trees,
        Object::Trees if lumber > 2 => Object::Lumberyard,
        Object::Lumberyard if lumber == 1 || trees == 0 => Object::OpenGround,
        object => object,
    }
}

fn advance_one_minute(previous: &Map, next: &mut Map) {
    for (y, row) in next.iter_mut().enumerate() {
        for (x, object) in row.iter_mut().enumerate() {
            *object = change(previous, x, y);
        }
    }
}

fn count(map: &Map) -> usize {
    let mut trees = 0;
    let mut lumber = 0;

    for row in map.iter() {
        for &object in row.iter() {
            match object {
                Object::Trees => trees += 1,
                Object::Lumberyard => lumber += 1,
                Object::OpenGround => (),
            }
        }
    }

    trees * lumber
}

fn advance_time_naive(map: &mut Map) -> usize {
    let mut next = &mut map.clone();
    let mut previous = map;
    for _ in 0..PART_ONE {
        advance_one_minute(previous, next);
        mem::swap(&mut previous, &mut next);
    }

    count(previous)
}

fn advance_time_oracle(map: &mut Map) -> usize {
    let mut next = &mut map.clone();
    let mut previous = map;
    for _ in PART_ONE..THRESHOLD {
        advance_one_minute(previous, next);

        mem::swap(&mut previous, &mut next);
    }

    let mut times = HashMap::new();
    let mut values = Vec::new();

    for t in THRESHOLD..PART_TWO {
        let value = count(previous);
        values.push(value);

        if let Some(tt) = times.insert(value, t) {
            return values[tt + (PART_TWO - t) % (t - tt) - THRESHOLD];
        }

        advance_one_minute(previous, next);

        mem::swap(&mut previous, &mut next);
    }

    count(previous)
}

fn main() -> Result<(), Error> {
    let path = Path::new("inputs/input-18-01.txt");

    let mut map = parse_input(path)?;
    println!("Part 1: {:?}", advance_time_naive(&mut map));
    println!("Part 2: {:?}", advance_time_oracle(&mut map));
    Ok(())
}
