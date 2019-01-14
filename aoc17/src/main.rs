use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader, Error as IoError};
use std::num::ParseIntError;
use std::ops::Range;
use std::path::Path;
use std::str::FromStr;

const SPRING: usize = 500;

#[derive(Debug)]
enum Error {
    Io(IoError),
    ParseInt(ParseIntError),
    Invalid(String),
}

impl From<IoError> for Error {
    fn from(error: IoError) -> Self {
        Error::Io(error)
    }
}

impl From<ParseIntError> for Error {
    fn from(error: ParseIntError) -> Self {
        Error::ParseInt(error)
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
            Error::ParseInt(e) => fmt::Display::fmt(e, f),
            Error::Invalid(s) => write!(f, "invalid input: {}", s),
        }
    }
}

enum Blueprints {
    Horizontal { x: Range<usize>, y: usize },
    Vertical { x: usize, y: Range<usize> },
}

fn parse_num(s: &str, range: Range<usize>) -> Result<usize, Error> {
    match s.get(range) {
        Some(s) => s.parse().map_err(Error::from),
        _ => Err(s.into()),
    }
}

impl FromStr for Blueprints {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let horizontal = match s.get(0..2) {
            Some("x=") => false,
            Some("y=") => true,
            _ => return Err(s.into()),
        };

        let (comma, dot) = match (s.find(','), s.find('.')) {
            (Some(c), Some(d)) if d > c => (c, d),
            _ => return Err(s.into()),
        };

        match (s.get(comma..comma + 4), s.get(dot..dot + 2), horizontal) {
            (Some(", x="), Some(".."), true) | (Some(", y="), Some(".."), false) => (),
            _ => return Err(s.into()),
        }

        let num = parse_num(s, 2..comma)?;
        let start = parse_num(s, comma + 4..dot)?;
        let end = parse_num(s, dot + 2..s.len())? + 1;
        let range = start..end;

        Ok(if horizontal {
            Blueprints::Horizontal { x: range, y: num }
        } else {
            Blueprints::Vertical { x: num, y: range }
        })
    }
}

fn parse_input(path: &Path) -> Result<Vec<Blueprints>, Error> {
    BufReader::new(File::open(path)?)
        .lines()
        .map(|line| line.map_err(Error::from).and_then(|s| s.parse()))
        .collect()
}

fn find_extremes(blueprints: &[Blueprints]) -> (usize, usize, usize, usize) {
    blueprints.iter().fold(
        (
            usize::max_value(),
            usize::max_value(),
            usize::min_value(),
            usize::min_value(),
        ),
        |(x_ming, y_ming, x_maxg, y_maxg), blueprint| {
            let (x_min, y_min, x_max, y_max) = match blueprint {
                Blueprints::Vertical {
                    x,
                    y: Range { start, end },
                } => (*x, *start, *x, *end - 1),
                Blueprints::Horizontal {
                    x: Range { start, end },
                    y,
                } => (*start, *y, *end - 1, *y),
            };

            (
                if x_min < x_ming { x_min } else { x_ming },
                if y_min < y_ming { y_min } else { y_ming },
                if x_max > x_maxg { x_max } else { x_maxg },
                if y_max > y_maxg { y_max } else { y_maxg },
            )
        },
    )
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
enum Object {
    Sand,
    Visited,
    Clay,
    Water,
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Object::Sand => write!(f, "."),
            Object::Visited => write!(f, "|"),
            Object::Clay => write!(f, "#"),
            Object::Water => write!(f, "~"),
        }
    }
}

struct Map {
    objects: Vec<Object>,
    transposed: Vec<Object>,
    depth: usize,
    width: usize,
}

impl fmt::Display for Map {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut chunks = self.transposed.chunks_exact(self.width);

        for &object in chunks.next().unwrap() {
            write!(f, "{}", object)?;
        }

        for chunk in chunks {
            writeln!(f)?;

            for &object in chunk {
                write!(f, "{}", object)?;
            }
        }

        Ok(())
    }
}

impl Map {
    fn new(depth: usize, width: usize) -> Self {
        let objects = vec![Object::Sand; depth * width];
        let transposed = objects.clone();

        Map {
            objects,
            transposed,
            depth,
            width,
        }
    }

    fn index_to_tuple(&self, index: usize) -> (usize, usize) {
        (index / self.depth, index % self.depth)
    }

    fn tuple_to_index(&self, tuple: (usize, usize)) -> usize {
        tuple.0 * self.depth + tuple.1
    }

    fn index_to_tuple_transposed(&self, index: usize) -> (usize, usize) {
        (index % self.width, index / self.width)
    }

    fn tuple_to_index_transposed(&self, tuple: (usize, usize)) -> usize {
        tuple.1 * self.width + tuple.0
    }

    fn fall_down(&mut self, x: usize, y: usize) {
        let start = self.tuple_to_index((x, y));
        let bottom = self.tuple_to_index((x, self.depth));

        if self.objects[start] != Object::Sand {
            return;
        }

        let end = self.objects[start..bottom]
            .iter()
            .cloned()
            .enumerate()
            .find_map(|(i, object)| match object {
                Object::Clay | Object::Water => Some(start + i),
                _ => None,
            });

        for object in self.objects[start..end.unwrap_or(bottom)].iter_mut() {
            *object = Object::Visited;
        }

        if let Some(end) = end {
            let (x, y) = self.index_to_tuple(end - 1);
            self.spread(x, y);
        }
    }

    fn spread(&mut self, x: usize, y: usize) {
        let left = self.tuple_to_index_transposed((0, y));
        let middle = self.tuple_to_index_transposed((x, y));
        let right = self.tuple_to_index_transposed((self.width, y));
        let under = self.tuple_to_index_transposed((x, y + 1));
        let end = self.tuple_to_index_transposed((self.width, y + 1));

        let (left_end, left_object) = self.transposed[left..middle]
            .iter()
            .cloned()
            .zip(self.transposed[right..under].iter().cloned())
            .enumerate()
            .rev()
            .find_map(|(i, (object, below))| match (object, below) {
                (Object::Clay, _)
                | (Object::Sand, Object::Sand)
                | (Object::Visited, Object::Sand) => Some((left + i + 1, object)),
                _ => None,
            })
            .unwrap();

        if left_object != Object::Visited {
            for object in self.transposed[left_end..middle].iter_mut() {
                *object = Object::Visited;
            }

            if left_object == Object::Sand {
                let (x, _) = self.index_to_tuple_transposed(left_end - 1);

                self.fall_down(x, y);
            }
        }

        let (right_end, right_object) = self.transposed[middle..right]
            .iter()
            .cloned()
            .zip(self.transposed[under..end].iter().cloned())
            .enumerate()
            .find_map(|(i, (object, below))| match (object, below) {
                (Object::Clay, _) | (Object::Water, _) => Some((middle + i, Object::Clay)),
                (Object::Sand, Object::Sand) | (Object::Visited, Object::Sand) => {
                    Some((middle + i, object))
                }
                _ => None,
            })
            .unwrap();

        if right_object != Object::Visited {
            for object in self.transposed[middle..right_end].iter_mut() {
                *object = Object::Visited;
            }

            if right_object == Object::Sand {
                let (x, _) = self.index_to_tuple_transposed(right_end);

                self.fall_down(x, y);
            }
        }

        if let (Object::Clay, Object::Clay) = (left_object, right_object) {
            for object in self.transposed[left_end..right_end].iter_mut() {
                *object = Object::Water;
            }
            return self.spread(x, y - 1);
        }
    }

    fn unite(&mut self) {
        let v: Vec<usize> = self
            .objects
            .iter()
            .cloned()
            .enumerate()
            .filter_map(|(i, object)| {
                if object == Object::Visited {
                    let index = self.tuple_to_index_transposed(self.index_to_tuple(i));

                    if self.transposed[index] == Object::Water {
                        None
                    } else {
                        Some(index)
                    }
                } else {
                    None
                }
            })
            .collect();

        for i in v {
            self.transposed[i] = Object::Visited;
        }
    }

    fn count(&self) -> (u32, u32) {
        self.transposed
            .iter()
            .cloned()
            .fold((0, 0), |(w, v), object| {
                if object == Object::Water {
                    (w + 1, v)
                } else if object == Object::Visited {
                    (w, v + 1)
                } else {
                    (w, v)
                }
            })
    }
}

fn populate_initial_state(blueprints: Vec<Blueprints>) -> (Map, usize) {
    let (x_min, y_min, x_max, y_max) = find_extremes(&blueprints);
    let width = x_max - x_min + 3;
    let depth = y_max - y_min + 1;
    let spring = SPRING - x_min + 1;
    let mut map = Map::new(depth, width);

    for tuple in blueprints.into_iter().flat_map(move |blueprint| {
        let (range, num, horizontal) = match blueprint {
            Blueprints::Vertical { x, y } => (y, x, false),
            Blueprints::Horizontal { x, y } => (x, y, true),
        };

        range.map(move |range| {
            if horizontal {
                (range - x_min + 1, num - y_min)
            } else {
                (num - x_min + 1, range - y_min)
            }
        })
    }) {
        let (t, o) = (
            map.tuple_to_index_transposed(tuple),
            map.tuple_to_index(tuple),
        );
        map.transposed[t] = Object::Clay;
        map.objects[o] = Object::Clay;
    }

    (map, spring)
}

fn main() -> Result<(), Error> {
    let path = Path::new("inputs/input-17-01.txt");

    let blueprints = parse_input(path)?;

    let (mut map, spring) = populate_initial_state(blueprints);

    map.fall_down(spring, 0);

    map.unite();

    let (water, visited) = map.count();

    println!("Part 1: {}", water + visited);
    println!("Part 2: {}", water);

    //    println!("{}", map);

    Ok(())
}
