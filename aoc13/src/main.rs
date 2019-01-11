use std::cmp::Ordering;
use std::collections::HashSet;
use std::fmt;
use std::fs::File;
use std::io::{Error as IoError, Read};
use std::path::Path;
use std::str::FromStr;

use self::Direction::*;
use self::NextTurn::*;
use self::Object::*;

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
            Error::Invalid(s) => write!(f, "invalid input: {}", s),
        }
    }
}

enum Object {
    Empty,
    Horizontal,
    Vertical,
    NWSEEdge,
    NESWEdge,
    Intersection,
}

#[derive(Clone)]
enum Direction {
    North,
    West,
    East,
    South,
}

#[derive(Clone)]
enum NextTurn {
    Left,
    Straight,
    Right,
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash)]
struct Position {
    y: usize,
    x: usize,
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{},{}", self.x, self.y)
    }
}

#[derive(Clone)]
struct Cart {
    position: Position,
    direction: Direction,
    next_turn: NextTurn,
}

impl PartialEq for Cart {
    fn eq(&self, other: &Self) -> bool {
        self.position.eq(&other.position)
    }
}

impl Eq for Cart {}

impl PartialOrd for Cart {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.position.partial_cmp(&other.position)
    }
}

impl Ord for Cart {
    fn cmp(&self, other: &Self) -> Ordering {
        self.position.cmp(&other.position)
    }
}

impl Cart {
    fn intersection(&mut self) -> &Direction {
        self.direction = match (&self.direction, &self.next_turn) {
            (West, Straight) | (North, Left) | (South, Right) => West,
            (East, Straight) | (North, Right) | (South, Left) => East,
            (North, Straight) | (West, Right) | (East, Left) => North,
            (South, Straight) | (West, Left) | (East, Right) => South,
        };

        self.next_turn = match self.next_turn {
            Left => Straight,
            Straight => Right,
            Right => Left,
        };

        &self.direction
    }
}

struct Track {
    objects: Vec<Object>,
    carts: Vec<Option<Cart>>,
    width: usize,
}

impl FromStr for Track {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = s.as_bytes();

        let width = match bytes.iter().enumerate().find(|&(_, &x)| x == b'\n') {
            Some((i, _)) => i,
            None => return Err(Error::Invalid(String::from("no newline found"))),
        };

        let mut objects = Vec::with_capacity(bytes.len());
        let mut carts = Vec::new();

        for (position, token) in bytes
            .chunks_exact(width + 1)
            .enumerate()
            .flat_map(|(y, row)| {
                row.iter()
                    .take(width)
                    .enumerate()
                    .map(move |(x, &token)| (Position { x, y }, token))
            })
        {
            objects.push(match token {
                b' ' => Empty,
                b'-' => Horizontal,
                b'|' => Vertical,
                b'/' => NWSEEdge,
                b'\\' => NESWEdge,
                b'+' => Intersection,
                b'^' => {
                    carts.push(Some(Cart {
                        position,
                        direction: North,
                        next_turn: Left,
                    }));

                    Vertical
                }
                b'v' => {
                    carts.push(Some(Cart {
                        position,
                        direction: South,
                        next_turn: Left,
                    }));

                    Vertical
                }
                b'<' => {
                    carts.push(Some(Cart {
                        position,
                        direction: West,
                        next_turn: Left,
                    }));

                    Horizontal
                }
                b'>' => {
                    carts.push(Some(Cart {
                        position,
                        direction: East,
                        next_turn: Left,
                    }));

                    Horizontal
                }
                inv => {
                    return Err(Error::Invalid(format!(
                        "invalid character: {} at position: ({}, {})",
                        char::from(inv),
                        position.x,
                        position.y
                    )))
                }
            });
        }

        Ok(Track {
            objects,
            carts,
            width,
        })
    }
}

fn parse_input(path: &Path) -> Result<Track, Error> {
    let mut s = String::new();
    File::open(path)?.read_to_string(&mut s)?;

    Ok(s.parse()?)
}

fn advance_single_cart(track: &mut Track, cart_id: usize) -> Option<Position> {
    let cart = track.carts[cart_id].as_mut()?;

    let object = track
        .objects
        .chunks_exact(track.width)
        .nth(cart.position.y)?
        .get(cart.position.x)?;

    match (object, &cart.direction, &cart.next_turn) {
        (Horizontal, West, _) => cart.position.x -= 1,
        (Horizontal, East, _) => cart.position.x += 1,
        (Vertical, North, _) => cart.position.y -= 1,
        (Vertical, South, _) => cart.position.y += 1,
        (NWSEEdge, North, _) | (NESWEdge, South, _) => {
            cart.position.x += 1;
            cart.direction = East;
        }
        (NWSEEdge, South, _) | (NESWEdge, North, _) => {
            cart.position.x -= 1;
            cart.direction = West;
        }
        (NWSEEdge, West, _) | (NESWEdge, East, _) => {
            cart.position.y += 1;
            cart.direction = South;
        }
        (NWSEEdge, East, _) | (NESWEdge, West, _) => {
            cart.position.y -= 1;
            cart.direction = North;
        }
        (Intersection, _, _) => match cart.intersection() {
            West => cart.position.x -= 1,
            East => cart.position.x += 1,
            North => cart.position.y -= 1,
            South => cart.position.y += 1,
        },
        _ => return None,
    };

    Some(cart.position)
}

fn simulate(mut track: Track) -> Option<()> {
    let mut set = HashSet::with_capacity(track.carts.len());

    for position in track
        .carts
        .iter()
        .map(|cart| cart.as_ref().unwrap().position)
    {
        set.insert(position);
    }

    loop {
        track.carts.sort_unstable();

        for i in 0..track.carts.len() {
            if track.carts[i] == None {
                continue;
            }
            set.remove(&track.carts[i].as_ref().unwrap().position);

            let position = advance_single_cart(&mut track, i)?;

            if !set.insert(position) {
                println!("Crash at: {}", position);
                set.remove(&position);

                for cart in track.carts.iter_mut() {
                    match cart {
                        Some(Cart { position: p, .. }) if position == *p => {
                            cart.take();
                        }
                        _ => (),
                    }
                }

                if set.len() == 1 {
                    let i = track
                        .carts
                        .iter()
                        .enumerate()
                        .find_map(|(i, cart)| cart.as_ref().map(|_| i))
                        .unwrap();

                    println!("Last cart at: {}", advance_single_cart(&mut track, i)?);

                    return Some(());
                }
            }
        }
    }
}

fn main() -> Result<(), Error> {
    let path = Path::new("inputs/input-13-01.txt");

    simulate(parse_input(path)?);

    Ok(())
}
