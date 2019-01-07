use std::fmt;
use std::fs::File;
use std::io::{prelude::*, BufReader, Error as IoError};
use std::path::Path;

use regex::Regex;

#[derive(Debug)]
enum Error {
    Io(IoError),
    Invalid,
}

struct Edges {
    min_x: i32,
    min_y: i32,
    max_x: i32,
    max_y: i32,
}

impl Edges {
    fn area(&self) -> (usize, usize) {
        (
            (self.max_x - self.min_x + 1) as usize,
            (self.max_y - self.min_y + 1) as usize,
        )
    }
}

struct Position {
    x: i32,
    y: i32,
}

impl Position {
    fn index(&self, edges: &Edges) -> usize {
        let row = (edges.max_x - edges.min_x + 2) as usize;

        let x = (self.x - edges.min_x) as usize;
        let y = (self.y - edges.min_y) as usize;

        y * row + x
    }
}

struct Velocity {
    x: i32,
    y: i32,
}

struct Signal {
    position: Position,
    velocity: Velocity,
}

impl Signal {
    fn at(&self, time: i32) -> Position {
        Position {
            x: self.position.x + self.velocity.x * time,
            y: self.position.y + self.velocity.y * time,
        }
    }
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

fn parse_input(path: &Path) -> Result<Vec<Signal>, Error> {
    let re = Regex::new(
        r"(?x)
        position=<\s*
        (?P<x>-?[0-9]+),\s+
        (?P<y>-?[0-9]+)>\s
        velocity=<\s*
        (?P<v_x>-?[0-9]+),\s+
        (?P<v_y>-?[0-9]+)>",
    )
    .unwrap();

    BufReader::new(File::open(path)?)
        .lines()
        .map(|line| {
            line.map_err(|e| e.into()).and_then(|s| {
                re.captures(&s).ok_or(Error::Invalid).map(|cap| Signal {
                    position: Position {
                        x: cap["x"].parse().unwrap(),
                        y: cap["y"].parse().unwrap(),
                    },
                    velocity: Velocity {
                        x: cap["v_x"].parse().unwrap(),
                        y: cap["v_y"].parse().unwrap(),
                    },
                })
            })
        })
        .collect()
}

fn find_min_area(signals: &[Signal]) -> (i32, Edges) {
    (0..)
        .scan(!0, |area_prev, t| {
            let edges = find_edges(signals, t);
            let (row, col) = edges.area();
            let area = row * col;

            if area < *area_prev {
                *area_prev = area;
                Some((t, edges))
            } else {
                None
            }
        })
        .last()
        .unwrap()
}

fn find_edges(signals: &[Signal], time: i32) -> Edges {
    let (min_x, min_y, max_x, max_y) = signals.iter().map(|signal| signal.at(time)).fold(
        (
            i32::max_value(),
            i32::max_value(),
            i32::min_value(),
            i32::min_value(),
        ),
        |(min_x, min_y, max_x, max_y), Position { x, y }| {
            (
                if x < min_x { x } else { min_x },
                if y < min_y { y } else { min_y },
                if x > max_x { x } else { max_x },
                if y > max_y { y } else { max_y },
            )
        },
    );

    Edges {
        min_x,
        min_y,
        max_x,
        max_y,
    }
}

fn draw(signals: &[Signal], time: i32, edges: &Edges) -> String {
    let (row, col) = edges.area();

    let mut v = Vec::with_capacity((row + 1) * col);

    for _ in 0..col {
        for _ in 0..row {
            v.push(b'.');
        }

        v.push(b'\n');
    }

    for signal in signals {
        v[signal.at(time).index(edges)] = b'#';
    }

    String::from_utf8(v).unwrap()
}

fn main() -> Result<(), Error> {
    let path = Path::new("inputs/input-10-01.txt");

    let signals = parse_input(path)?;

    let (t, edges) = find_min_area(&signals);

    print!("Part 1:\n{}", draw(&signals, t, &edges));
    println!("Part 2: {}", t);

    Ok(())
}
