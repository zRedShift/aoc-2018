use std::fs::File;
use std::io::{prelude::*, BufReader, Error};

use std::path::Path;

fn parse_input(path: &Path) -> Result<Vec<u8>, Error> {
    let f = File::open(path)?;

    BufReader::new(f)
        .bytes()
        .filter(|result| match result {
            Ok(x) if !x.is_ascii_alphabetic() => false,
            _ => true,
        })
        .collect()
}

fn opposite_case(a: u8, b: u8) -> bool {
    if a > b {
        a - b == 0x20
    } else {
        b - a == 0x20
    }
}

fn react(poly: &[u8], buffer: &mut Vec<u8>, skip: u8) {
    for &unit in poly {
        if skip == unit || skip == unit + 0x20 {
            continue;
        }

        match buffer.last() {
            Some(&last) if opposite_case(unit, last) => {
                buffer.pop();
            }
            _ => buffer.push(unit),
        };
    }
}

fn part_one(poly: &[u8]) -> usize {
    let buffer = &mut vec![];

    react(poly, buffer, 0);

    buffer.len()
}

fn part_two(poly: &[u8]) -> usize {
    let buffer = &mut vec![];

    (b'a'..=b'z')
        .map(|skip| {
            buffer.truncate(0);
            react(poly, buffer, skip);

            buffer.len()
        })
        .min()
        .unwrap()
}

fn main() -> Result<(), Error> {
    let path = Path::new("inputs/input-05-01.txt");

    let poly = parse_input(path)?;

    println!("Part 1: {:?}", part_one(&poly));
    println!("Part 2: {:?}", part_two(&poly));

    Ok(())
}
