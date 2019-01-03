use std::fmt;
use std::fs::File;
use std::io::{prelude::*, BufReader, Error as IoError};
use std::iter;
use std::path::Path;

const MAX_TOTAL: u32 = 10000;

#[derive(Debug)]
enum Error {
    Io(IoError),
    Invalid,
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Self {
        Error::Io(e)
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

fn parse_input(path: &Path) -> Result<Vec<(u32, u32)>, Error> {
    BufReader::new(File::open(path)?)
        .lines()
        .map(|line| {
            line.map_err(|e| e.into()).and_then(|s| {
                let mut iter = s.split(", ").map(|s| s.parse());

                match (iter.next(), iter.next()) {
                    (Some(Ok(x)), Some(Ok(y))) => Ok((x, y)),
                    _ => Err(Error::Invalid),
                }
            })
        })
        .collect()
}

fn bounds(coordinates: &[(u32, u32)]) -> (u32, u32, u32, u32) {
    coordinates
        .iter()
        .fold((!0, !0, 0, 0), |(x_min, y_min, x_max, y_max), &(x, y)| {
            (
                if x < x_min { x } else { x_min },
                if y < y_min { y } else { y_min },
                if x > x_max { x } else { x_max },
                if y > y_max { y } else { y_max },
            )
        })
}

fn abs(a: u32, b: u32) -> u32 {
    if a > b {
        a - b
    } else {
        b - a
    }
}

fn distance(a: (u32, u32), b: (u32, u32)) -> u32 {
    abs(a.0, b.0) + abs(a.1, b.1)
}

fn fold_min(accumulator: (Option<usize>, u32), input: (usize, u32)) -> (Option<usize>, u32) {
    let (id, min) = accumulator;
    let (i, dist) = input;

    if dist < min {
        (Some(i), dist)
    } else if dist == min {
        (None, min)
    } else {
        (id, min)
    }
}

fn find_min(coordinates: &[(u32, u32)], position: (u32, u32)) -> Option<usize> {
    let (id, _) = coordinates
        .iter()
        .map(|&z| distance(position, z))
        .enumerate()
        .fold((None, !0), fold_min);

    id
}

fn part_one(coordinates: &[(u32, u32)]) -> u32 {
    let (x_min, y_min, x_max, y_max) = bounds(coordinates);

    let mut counter = vec![0u32; coordinates.len()];

    (y_min + 1..y_max)
        .flat_map(|y| (x_min + 1..x_max).filter_map(move |x| find_min(coordinates, (x, y))))
        .for_each(|id| counter[id] += 1);

    (x_min..x_max)
        .zip(iter::repeat(y_min))
        .chain(iter::repeat(x_max).zip(y_min..=y_max))
        .chain(
            (x_min..x_max)
                .zip(iter::repeat(y_max))
                .chain(iter::repeat(x_min).zip(y_min + 1..y_max)),
        )
        .filter_map(|position| find_min(coordinates, position))
        .for_each(|id| counter[id] = 0);

    *counter.iter().max().unwrap()
}

fn total_distance(coordinates: &[(u32, u32)], position: (u32, u32)) -> u32 {
    coordinates.iter().map(|&z| distance(position, z)).sum()
}

fn part_two(coordinates: &[(u32, u32)]) -> usize {
    let (x_min, y_min, x_max, y_max) = bounds(coordinates);

    (y_min..=y_max)
        .flat_map(|y| {
            (x_min..=x_max).filter(move |&x| total_distance(coordinates, (x, y)) < MAX_TOTAL)
        })
        .count()
}

fn main() -> Result<(), Error> {
    let path = Path::new("inputs/input-06-01.txt");

    let coordinates = parse_input(path)?;

    println!("Part 1: {}", part_one(&coordinates));
    println!("Part 2: {}", part_two(&coordinates));

    Ok(())
}
