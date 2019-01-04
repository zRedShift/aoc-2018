use std::collections::HashSet;
use std::fmt;
use std::fs::File;
use std::io::{prelude::*, BufReader, Error as IoError};
use std::iter;
use std::path::Path;

use regex::Regex;

const ALPHABET_SIZE: usize = 26;
const BASE_TIME: usize = 61;
const WORKER_POOL: usize = 5;

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

fn parse_input(path: &Path) -> Result<(Vec<Vec<usize>>, Vec<Option<HashSet<usize>>>), Error> {
    let re = Regex::new(
        r"Step (?P<blocker>[A-Z]) must be finished before step (?P<waiter>[A-Z]) can begin.",
    )
    .unwrap();

    let mut waiters: Vec<_> = iter::repeat(Vec::new()).take(ALPHABET_SIZE).collect();
    let mut blockers: Vec<_> = iter::repeat(None).take(ALPHABET_SIZE).collect();

    let errors = BufReader::new(File::open(path)?)
        .lines()
        .map(|line| {
            line.map_err(|e| e.into())
                .and_then(|s| match re.captures(&s) {
                    Some(cap) => {
                        let blocker = (cap["blocker"].as_bytes()[0] - b'A') as usize;
                        let waiter = (cap["waiter"].as_bytes()[0] - b'A') as usize;
                        waiters[blocker].push(waiter);
                        blockers[blocker].get_or_insert(HashSet::new());
                        blockers[waiter]
                            .get_or_insert(HashSet::new())
                            .insert(blocker);
                        Ok(())
                    }
                    None => Err(Error::Invalid),
                })
        })
        .find(|r| r.is_err());

    match errors {
        Some(Err(e)) => Err(e),
        _ => Ok((waiters, blockers)),
    }
}

fn try_next(blockers: &mut [Option<HashSet<usize>>]) -> Option<usize> {
    blockers
        .iter_mut()
        .enumerate()
        .find(|(_, blocker)| match blocker {
            Some(e) => e.is_empty(),
            None => false,
        })
        .map(|(i, _)| i)
}

fn extract(i: usize, waiters: &[Vec<usize>], blockers: &mut [Option<HashSet<usize>>]) -> char {
    for &j in waiters[i].iter() {
        blockers[j].as_mut().unwrap().remove(&i);
    }

    (i as u8 + b'A') as char
}

fn part_one(waiters: &[Vec<usize>], blockers: &mut [Option<HashSet<usize>>]) -> String {
    let mut output = String::with_capacity(6);

    for _ in 0..ALPHABET_SIZE {
        let i = try_next(blockers).unwrap();

        blockers[i] = None;

        output.push(extract(i, waiters, blockers));
    }

    output
}

fn part_two(waiters: &[Vec<usize>], blockers: &mut [Option<HashSet<usize>>]) -> (String, usize) {
    let mut output = String::with_capacity(ALPHABET_SIZE);

    let mut workers = [None; WORKER_POOL];

    let mut time = 0;

    loop {
        let (free, t) = workers
            .iter_mut()
            .enumerate()
            .map(|(i, worker)| match *worker {
                Some((id, t)) if t <= time => {
                    *worker = None;
                    output.push(extract(id, waiters, blockers));
                    (i, 0)
                }
                Some((_, t)) => (i, t - time),
                None => (i, 0),
            })
            .fold((None, !0), |(free, time), (i, t)| {
                if t == 0 {
                    (Some(i), time)
                } else if time > t {
                    (free, t)
                } else {
                    (free, time)
                }
            });

        match (free, try_next(blockers)) {
            (None, _) | (_, None) => {
                if t == !0 {
                    break;
                }
                time += t
            }
            (Some(free), Some(next)) => {
                blockers[next] = None;
                workers[free] = Some((next, time + BASE_TIME + next))
            }
        }
    }

    (output, time)
}

fn main() -> Result<(), Error> {
    let path = Path::new("inputs/input-07-01.txt");

    let (waiters, mut blockers) = parse_input(path)?;

    println!("Part 1: {}", part_one(&waiters, &mut blockers.clone()));
    println!("Part 2: {:?}", part_two(&waiters, &mut blockers));

    Ok(())
}
