use std::collections::HashSet;
use std::fs::File;
use std::io::{Error, Read};
use std::path::Path;

fn parse_input(path: &Path) -> Result<String, Error> {
    let mut s = String::new();

    File::open(path)?.read_to_string(&mut s)?;

    Ok(s)
}

fn part_one(s: &str) -> i32 {
    let (mut two, mut three) = (0, 0);

    let mut frequency: [u8; 256];

    for s in s.lines() {
        frequency = [0u8; 256];

        for c in s.bytes().map(|c| c as usize) {
            frequency[c] = frequency[c].saturating_add(1);
        }

        if frequency.iter().any(|&n| n == 2) {
            two += 1;
        }

        if frequency.iter().any(|&n| n == 3) {
            three += 1;
        }
    }

    two * three
}

fn part_two(s: &str) -> Option<String> {
    let len = s.lines().next()?.len();

    for i in 0..len {
        let mut h = HashSet::new();

        for s in s.lines() {
            let key = String::with_capacity(len - 1) + &s[..i] + &s[i + 1..];

            if let Some(key) = h.replace(key) {
                return Some(key);
            }
        }
    }

    None
}

fn main() -> Result<(), Error> {
    let path = Path::new("inputs/input-02-01.txt");

    let s = parse_input(path)?;

    println!("Part 1: {}", part_one(&s));
    println!("Part 2: {:?}", part_two(&s));

    Ok(())
}
