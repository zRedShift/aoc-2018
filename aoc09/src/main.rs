use std::fmt;
use std::fs::File;
use std::io::{prelude::*, Error as IoError};
use std::iter;
use std::mem;
use std::path::Path;

use regex::Regex;

struct Marble {
    next: usize,
    prev: usize,
}

struct Circle {
    marbles: Vec<Marble>,
    current: usize,
}

impl Circle {
    fn new(marbles: usize) -> Self {
        let mut marbles = Vec::with_capacity(marbles);

        marbles.push(Marble { next: 0, prev: 0 });

        Circle {
            marbles,
            current: 0,
        }
    }

    fn play(&mut self) -> (usize, usize) {
        let new = self.marbles.len();

        if new % 23 == 0 {
            self.marbles.push(Marble { next: 0, prev: 0 });
            (new, new + self.remove())
        } else {
            self.insert(new);

            (new, 0)
        }
    }

    fn insert(&mut self, new: usize) {
        let prev = self.marbles[self.current].next;

        let next = mem::replace(&mut self.marbles[prev].next, new);

        self.marbles.push(Marble { next, prev });

        self.marbles[next].prev = new;

        self.current = new;
    }

    fn remove(&mut self) -> usize {
        let removed = (0..7).fold(self.current, |x, _| self.marbles[x].prev);

        let Marble { next, prev } = self.marbles[removed];

        self.marbles[next].prev = prev;
        self.marbles[prev].next = next;

        self.current = next;

        removed
    }
}

#[derive(Debug)]
enum Error {
    Io(IoError),
    Invalid,
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
            Error::Invalid => write!(f, "invalid input"),
        }
    }
}

fn parse_input(path: &Path) -> Result<(usize, usize), Error> {
    let s = &mut String::new();

    File::open(path)?.read_to_string(s)?;

    let caps =
        Regex::new("(?P<players>[0-9]+) players; last marble is worth (?P<marbles>[0-9]+) points")
            .unwrap()
            .captures(s)
            .ok_or(Error::Invalid)?;

    Ok((
        caps["players"].parse().unwrap(),
        caps["marbles"].parse().unwrap(),
    ))
}

fn play(players: usize, marbles: usize) -> usize {
    let mut scores = vec![0; players];
    let mut circle = Circle::new(marbles);

    iter::repeat_with(|| circle.play())
        .take(marbles)
        .map(|(marble, score)| {
            let player = marble % players;
            scores[player] += score;

            scores[player]
        })
        .max()
        .unwrap()
}

fn main() -> Result<(), Error> {
    let path = Path::new("inputs/input-09-01.txt");

    let (players, marbles) = parse_input(path)?;

    println!("Part 1: {}", play(players, marbles));
    println!("Part 2: {}", play(players, marbles * 100));

    Ok(())
}
