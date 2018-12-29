use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

fn parse_input(path: &Path) -> Result<Vec<i32>, Box<dyn Error>> {
    let f = File::open(path)?;

    BufReader::new(f)
        .lines()
        .map(|line| {
            line.map_err(|e| e.into())
                .and_then(|s| s.parse::<i32>().map_err(|e| e.into()))
        })
        .collect()
}

fn part_one(numbers: &[i32]) -> i32 {
    numbers.iter().sum()
}

fn part_two(numbers: &[i32]) -> i32 {
    let mut set = HashSet::new();
    let mut sum = 0;

    loop {
        for &num in numbers {
            if !set.insert(sum) {
                return sum;
            }

            sum += num
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let path = Path::new("inputs/input-01-01.txt");

    let values = parse_input(path)?;

    println!("Part 1: {}", part_one(&values));

    println!("Part 2: {}", part_two(&values));

    Ok(())
}
