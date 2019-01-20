use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader, Error as IoError};
use std::num::ParseIntError;
use std::path::Path;
use std::str::FromStr;

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

#[derive(Debug)]
struct Coordinate {
    x: i32,
    y: i32,
    z: i32,
}

impl Coordinate {
    fn distance(&self, other: &Self) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs() + (self.z - other.z).abs()
    }
}

struct Octant {
    x_min: i32,
    x_max: i32,
    y_min: i32,
    y_max: i32,
    z_min: i32,
    z_max: i32,
}

type Octets = [Octant; 8];

impl Octant {
    fn octets(&self) -> Octets {
        let Octant {
            x_min,
            x_max,
            y_min,
            y_max,
            z_min,
            z_max,
        } = *self;
        let x_mid = (x_max + x_min) / 2;
        let y_mid = (y_max + y_min) / 2;
        let z_mid = (z_max + z_min) / 2;

        [
            Octant {
                x_min,
                x_max: x_mid,
                y_min,
                y_max: y_mid,
                z_min,
                z_max: z_mid,
            },
            Octant {
                x_min: x_mid,
                x_max,
                y_min,
                y_max: y_mid,
                z_min,
                z_max: z_mid,
            },
            Octant {
                x_min,
                x_max: x_mid,
                y_min: y_mid,
                y_max,
                z_min,
                z_max: z_mid,
            },
            Octant {
                x_min: x_mid,
                x_max,
                y_min: y_mid,
                y_max,
                z_min,
                z_max: z_mid,
            },
            Octant {
                x_min,
                x_max: x_mid,
                y_min,
                y_max: y_mid,
                z_min: z_mid,
                z_max,
            },
            Octant {
                x_min: x_mid,
                x_max,
                y_min,
                y_max: y_mid,
                z_min: z_mid,
                z_max,
            },
            Octant {
                x_min,
                x_max: x_mid,
                y_min: y_mid,
                y_max,
                z_min: z_mid,
                z_max,
            },
            Octant {
                x_min: x_mid,
                x_max,
                y_min: y_mid,
                y_max,
                z_min: z_mid,
                z_max,
            },
        ]
    }

    fn coverage(&self, nanobot: &NanoBot) -> bool {
        nanobot.in_radius(&Coordinate {
            x: if nanobot.coordinate.x < self.x_min {
                self.x_min
            } else if nanobot.coordinate.x > self.x_max {
                self.x_max
            } else {
                nanobot.coordinate.x
            },
            y: if nanobot.coordinate.y < self.y_min {
                self.y_min
            } else if nanobot.coordinate.y > self.y_max {
                self.y_max
            } else {
                nanobot.coordinate.y
            },
            z: if nanobot.coordinate.z < self.z_min {
                self.z_min
            } else if nanobot.coordinate.z > self.z_max {
                self.z_max
            } else {
                nanobot.coordinate.z
            },
        })
    }

    fn coverage_all(&self, nanobots: &[NanoBot]) -> i32 {
        nanobots
            .iter()
            .filter(|&nanobot| self.coverage(nanobot))
            .count() as i32
    }

    fn size(&self) -> i32 {
        (self.x_max - self.x_min)
            .saturating_mul((self.y_max - self.y_min).saturating_mul(self.y_max - self.y_min))
    }

    fn expand(&mut self, add: i32) {
        self.x_min -= add;
        self.x_max += add;
        self.y_min -= add;
        self.y_max += add;
        self.z_min -= add;
        self.z_max += add;
    }
}

struct NanoBot {
    coordinate: Coordinate,
    radius: i32,
}

impl NanoBot {
    fn in_radius(&self, other: &Coordinate) -> bool {
        self.radius >= self.coordinate.distance(other)
    }
}

impl FromStr for NanoBot {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string.get(0..5) {
            Some("pos=<") => (),
            Some(invalid) => return Err(invalid.into()),
            None => return Err("unexpected EOL".into()),
        }

        let mut split = string[5..].split(',');

        let x = split
            .next()
            .ok_or_else(|| "x coordinate missing".into())
            .and_then(|s| s.parse().map_err(Error::from))?;

        let y = split
            .next()
            .ok_or_else(|| "y coordinate missing".into())
            .and_then(|s| s.parse().map_err(Error::from))?;

        let z = split
            .next()
            .ok_or_else(|| "z coordinate missing".into())
            .and_then(|s| {
                match s.bytes().last() {
                    Some(b'>') => (),
                    Some(s) => return Err(format!("{}", s as char).into()),
                    None => return Err("z coordinate missing".into()),
                }

                s[0..s.len() - 1].parse().map_err(Error::from)
            })?;

        let coordinate = Coordinate { x, y, z };

        let radius = split
            .next()
            .ok_or_else(|| "radius missing".into())
            .and_then(|s| {
                match s.get(0..3) {
                    Some(" r=") => (),
                    Some(s) => return Err(s.into()),
                    None => return Err("radius missing".into()),
                }

                s[3..].parse().map_err(Error::from)
            })?;

        match split.next() {
            None => Ok(NanoBot { coordinate, radius }),
            Some(junk) => Err(junk.into()),
        }
    }
}

fn parse_input(path: &Path) -> Result<Vec<NanoBot>, Error> {
    let nanobots = BufReader::new(File::open(path)?)
        .lines()
        .map(|line| line.map_err(Error::from).and_then(|s| s.parse()))
        .collect::<Result<Vec<NanoBot>, Error>>()?;

    if !nanobots.is_empty() {
        Ok(nanobots)
    } else {
        Err("empty input".into())
    }
}

fn largest_radius(nanobots: &[NanoBot]) -> &NanoBot {
    nanobots
        .iter()
        .max_by_key(|NanoBot { radius, .. }| radius)
        .unwrap()
}

fn count_in_radius(nanobots: &[NanoBot], nanobot: &NanoBot) -> usize {
    nanobots
        .iter()
        .filter(|other| nanobot.in_radius(&other.coordinate))
        .count()
}

fn enclosing_volume(nanobots: &[NanoBot]) -> Octant {
    nanobots.iter().fold(
        Octant {
            x_min: i32::max_value(),
            x_max: i32::min_value(),
            y_min: i32::max_value(),
            y_max: i32::min_value(),
            z_min: i32::max_value(),
            z_max: i32::min_value(),
        },
        |Octant {
             x_min,
             x_max,
             y_min,
             y_max,
             z_min,
             z_max,
         },
         &NanoBot {
             coordinate: Coordinate { x, y, z },
             ..
         }| {
            Octant {
                x_min: if x < x_min { x } else { x_min },
                x_max: if x >= x_max { x + 1 } else { x_max },
                y_min: if y < y_min { y } else { y_min },
                y_max: if y >= y_max { y + 1 } else { y_max },
                z_min: if z < z_min { z } else { z_min },
                z_max: if z >= z_max { z + 1 } else { z_max },
            }
        },
    )
}

fn distance(nanobots: &[NanoBot], coordinate: &Coordinate, max_in_range: i32) -> Option<i32> {
    let in_range = nanobots
        .iter()
        .filter(|nanobot| nanobot.in_radius(coordinate))
        .count() as i32;
    if in_range == max_in_range {
        Some(coordinate.x + coordinate.y + coordinate.z)
    } else {
        None
    }
}

fn search(nanobots: &[NanoBot], octant: &Octant, in_range: i32) -> Option<i32> {
    match octant.size() {
        0 => None,
        1 => distance(
            nanobots,
            &Coordinate {
                x: octant.x_min,
                y: octant.y_min,
                z: octant.z_min,
            },
            in_range,
        ),
        _ => {
            if octant.coverage_all(nanobots) < in_range {
                None
            } else {
                octant
                    .octets()
                    .iter()
                    .filter_map(|octant| search(nanobots, octant, in_range))
                    .min()
            }
        }
    }
}

fn init_search(nanobots: &[NanoBot]) -> (i32, i32) {
    let max_in_range = nanobots.len() as i32 + 1;
    let mut octant = enclosing_volume(&nanobots);
    octant.expand(largest_radius(nanobots).radius);
    for in_range in (0..max_in_range).rev() {
        if let Some(distance) = search(nanobots, &octant, in_range) {
            return (in_range, distance);
        }
    }

    (0, 0)
}

fn main() -> Result<(), Error> {
    let path = Path::new("inputs/input-23-01.txt");

    let nanobots = parse_input(path)?;

    println!(
        "Part 1: {}",
        count_in_radius(&nanobots, largest_radius(&nanobots))
    );

    let (in_range, distance) = init_search(&nanobots);
    println!("Part 2: {} ({} in range)", distance, in_range);

    Ok(())
}
