use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::{Error as IoError, Read};
use std::path::Path;

#[macro_use]
extern crate bitflags;

const THRESHOLD: u16 = 999;

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

impl From<&str> for Error {
    fn from(error: &str) -> Self {
        Error::Invalid(error.into())
    }
}

impl From<String> for Error {
    fn from(error: String) -> Self {
        Error::Invalid(error)
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

bitflags! {
    struct Doors: u8 {
        const NONE = 0b0000;
        const NORTH = 0b0001;
        const WEST = 0b0010;
        const EAST = 0b0100;
        const SOUTH = 0b1000;
    }
}

#[derive(Debug)]
struct Map {
    doors: Vec<Doors>,
    stride: usize,
    start: usize,
}

fn populate(input: &[u8]) -> Result<Map, Error> {
    let mut map = HashMap::new();
    let mut locations = vec![];

    let (_, x_min, y_min, x_max, y_max) = input.iter().cloned().try_fold(
        ((0, 0), 0, 0, 0, 0),
        |(location, x_min, y_min, x_max, y_max), b| {
            let (x, y) = {
                match b {
                    b'N' => {
                        map.entry(location)
                            .and_modify(|o| *o |= Doors::NORTH)
                            .or_insert(Doors::NORTH);
                        let location = (location.0, location.1 - 1);
                        map.entry(location)
                            .and_modify(|o| *o |= Doors::SOUTH)
                            .or_insert(Doors::SOUTH);
                        location
                    }
                    b'W' => {
                        map.entry(location)
                            .and_modify(|o| *o |= Doors::WEST)
                            .or_insert(Doors::WEST);
                        let location = (location.0 - 1, location.1);
                        map.entry(location)
                            .and_modify(|o| *o |= Doors::EAST)
                            .or_insert(Doors::EAST);
                        location
                    }
                    b'E' => {
                        map.entry(location)
                            .and_modify(|o| *o |= Doors::EAST)
                            .or_insert(Doors::EAST);
                        let location = (location.0 + 1, location.1);
                        map.entry(location)
                            .and_modify(|o| *o |= Doors::WEST)
                            .or_insert(Doors::WEST);
                        location
                    }
                    b'S' => {
                        map.entry(location)
                            .and_modify(|o| *o |= Doors::SOUTH)
                            .or_insert(Doors::SOUTH);
                        let location = (location.0, location.1 + 1);
                        map.entry(location)
                            .and_modify(|o| *o |= Doors::NORTH)
                            .or_insert(Doors::NORTH);
                        location
                    }
                    b'(' => {
                        locations.push(location);
                        return Ok((location, x_min, y_min, x_max, y_max));
                    }
                    b')' => {
                        return match locations.pop() {
                            Some(location) => Ok((location, x_min, y_min, x_max, y_max)),
                            None => Err("no open parentheses to close".into()),
                        }
                    }
                    b'|' => {
                        return match locations.last() {
                            Some(&location) => Ok((location, x_min, y_min, x_max, y_max)),
                            None => Ok(((0, 0), x_min, y_min, x_max, y_max)),
                        }
                    }
                    invalid => {
                        return Err(Error::Invalid(format!(
                            "invalid character: {}",
                            invalid as char
                        )))
                    }
                }
            };

            Ok((
                (x, y),
                if x < x_min { x } else { x_min },
                if y < y_min { y } else { y_min },
                if x > x_max { x } else { x_max },
                if y > y_max { y } else { y_max },
            ))
        },
    )?;

    if !locations.is_empty() {
        return Err("unclosed parentheses".into());
    }

    let x_size = x_max - x_min + 1;
    let y_size = y_max - y_min + 1;
    let stride = x_size as usize;
    let start = (0 - y_min * x_size - x_min) as usize;
    let mut doors = vec![Doors::NONE; stride * y_size as usize];

    for ((x, y), door) in map.drain() {
        doors[((y - y_min) * x_size + x - x_min) as usize] = door;
    }

    Ok(Map {
        doors,
        stride,
        start,
    })
}

fn parse_input(path: &Path) -> Result<Map, Error> {
    let mut buffer = Vec::new();
    File::open(path)?.read_to_end(&mut buffer)?;

    let data = match (
        buffer.first(),
        buffer.iter().enumerate().rev().find(|(_, &b)| b != b'\n'),
    ) {
        (Some(b'^'), Some((len, b'$'))) => &buffer[1..len],
        _ => return Err("invalid regex syntax".into()),
    };

    populate(data)
}

fn calculate_distances(map: &Map, distances: &mut [u16], position: usize, distance: u16) {
    distances[position] = distance;

    let doors = map.doors[position];
    let new_dist = distance + 1;

    if doors.contains(Doors::NORTH) && distances[position - map.stride] > new_dist {
        calculate_distances(map, distances, position - map.stride, new_dist);
    }
    if doors.contains(Doors::WEST) && distances[position - 1] > new_dist {
        calculate_distances(map, distances, position - 1, new_dist);
    }
    if doors.contains(Doors::EAST) && distances[position + 1] > new_dist {
        calculate_distances(map, distances, position + 1, new_dist);
    }
    if doors.contains(Doors::SOUTH) && distances[position + map.stride] > new_dist {
        calculate_distances(map, distances, position + map.stride, new_dist);
    }
}

fn longest_shortest_path(map: &Map) -> (u16, usize) {
    let mut distances = vec![u16::max_value(); map.doors.len()];

    calculate_distances(&map, &mut distances, map.start, 0);

    (
        distances.iter().cloned().max().unwrap(),
        distances.iter().cloned().filter(|&x| x > THRESHOLD).count(),
    )
}

fn main() -> Result<(), Error> {
    let path = Path::new("inputs/input-20-01.txt");

    let map = parse_input(path)?;

    let (longest, count) = longest_shortest_path(&map);

    println!("Part 1: {}", longest);
    println!("Part 2: {}", count);

    Ok(())
}
