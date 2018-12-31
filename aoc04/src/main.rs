use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use regex::Regex;

struct Distribution {
    histogram: [u8; 1 << 6],
    total: usize,
}

enum Entry {
    GuardId(usize),
    FallAsleep(usize),
    WakeUp(usize),
}

fn parse_input(path: &Path) -> Result<HashMap<usize, Distribution>, Box<dyn Error>> {
    let f = File::open(path)?;

    let regex = Regex::new(
        r"(?x)
            \[
            \d{4}-\d{2}-\d{2}
            \s\d{2}:(?P<minute>\d{2})
            ]\s
            (?P<entry>wakes\sup
            |falls\sasleep
            |Guard\s\#(?P<guard>\d+))",
    )
    .unwrap();

    let ordered: BTreeMap<String, Entry> = BufReader::new(f)
        .lines()
        .map(|line| {
            line.map_err(|e| e.into()).and_then(|s| {
                match regex.captures(&s) {
                    None => Err(Box::<Error>::from("Invalid pattern.")),
                    Some(cap) => match &cap["entry"] {
                        "falls asleep" => Ok(Entry::FallAsleep(cap["minute"].parse().unwrap())),
                        "wakes up" => Ok(Entry::WakeUp(cap["minute"].parse().unwrap())),
                        _ => Ok(Entry::GuardId(cap["guard"].parse().unwrap())),
                    },
                }
                .map(|entry| (s, entry))
            })
        })
        .collect::<Result<_, _>>()?;

    let mut h = HashMap::new();

    ordered
        .into_iter()
        .fold((0, 0), |(id, start), (_, entry)| match entry {
            Entry::GuardId(id) => (id, 0),
            Entry::FallAsleep(start) => (id, start),
            Entry::WakeUp(end) => {
                let duration = end - start;

                let dist = h.entry(id).or_insert(Distribution {
                    histogram: [0u8; 1 << 6],
                    total: 0,
                });

                dist.total += duration;

                for x in dist.histogram.iter_mut().skip(start).take(duration) {
                    *x = x.saturating_add(1);
                }

                (id, 0)
            }
        });

    Ok(h)
}

fn part_one(h: &HashMap<usize, Distribution>) -> Option<usize> {
    h.iter()
        .max_by_key(|&(_, &Distribution { total, .. })| total)
        .map(|(&id, &Distribution { histogram, .. })| {
            histogram
                .iter()
                .enumerate()
                .max_by_key(|&(_, &val)| val)
                .map(|(a, _)| a)
                .unwrap()
                * id
        })
}

fn part_two(h: &HashMap<usize, Distribution>) -> Option<usize> {
    h.iter()
        .map(|(&id, &Distribution { histogram, .. })| {
            let (minute, &frequency) = histogram
                .iter()
                .enumerate()
                .max_by_key(|&(_, &val)| val)
                .unwrap();

            (id, minute, frequency)
        })
        .max_by_key(|&(_, _, frequency)| frequency)
        .map(|(id, minute, _)| id * minute)
}

fn main() -> Result<(), Box<dyn Error>> {
    let path = Path::new("inputs/input-04-01.txt");

    let h = parse_input(path)?;

    println!("Part 1: {:?}", part_one(&h));
    println!("Part 2: {:?}", part_two(&h));

    Ok(())
}
