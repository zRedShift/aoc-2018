use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use regex::Regex;

const BOARD_SIZE: usize = 1 << 10;

struct Offset(usize, usize);

struct Size(usize, usize);

struct Rect {
    id: u32,
    offset: Offset,
    size: Size,
}

#[derive(Debug)]
struct PatternError;

impl Display for PatternError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Invalid pattern.")
    }
}

impl Error for PatternError {}

fn parse_input(path: &Path) -> Result<Vec<Rect>, Box<dyn Error>> {
    let re = Regex::new(r"#(\d{1,4}) @ (\d{1,3}),(\d{1,3}): (\d{1,3})x(\d{1,3})").unwrap();

    let f = File::open(path)?;

    BufReader::new(f)
        .lines()
        .map(|line| {
            line.map_err(|e| e.into()).and_then(|s| {
                re.captures(&s)
                    .ok_or_else(|| PatternError)
                    .and_then(
                        |c| match (c.get(1), c.get(2), c.get(3), c.get(4), c.get(5)) {
                            (Some(id), Some(off_x), Some(off_y), Some(size_x), Some(size_y)) => {
                                Ok(Rect {
                                    id: id.as_str().parse().unwrap(),
                                    offset: Offset(
                                        off_x.as_str().parse().unwrap(),
                                        off_y.as_str().parse().unwrap(),
                                    ),
                                    size: Size(
                                        size_x.as_str().parse().unwrap(),
                                        size_y.as_str().parse().unwrap(),
                                    ),
                                })
                            }
                            _ => Err(PatternError),
                        },
                    )
                    .map_err(|e| e.into())
            })
        })
        .collect()
}

fn part_one(rects: &[Rect]) -> ([[u8; BOARD_SIZE]; BOARD_SIZE], u32) {
    let mut board = [[0u8; BOARD_SIZE]; BOARD_SIZE];

    let collisions: usize = rects
        .iter()
        .map(|rect| {
            board
                .iter_mut()
                .skip(rect.offset.1)
                .take(rect.size.1)
                .flat_map(|row| {
                    row.iter_mut()
                        .skip(rect.offset.0)
                        .take(rect.size.0)
                        .map(|cell| {
                            *cell = cell.saturating_add(1);
                            *cell
                        })
                        .filter(|&cell| cell == 2)
                })
                .count()
        })
        .sum();

    (board, collisions as u32)
}

fn part_two(rects: &[Rect], board: [[u8; BOARD_SIZE]; BOARD_SIZE]) -> Option<u32> {
    rects
        .iter()
        .find(|&rect| {
            board
                .iter()
                .skip(rect.offset.1)
                .take(rect.size.1)
                .map(move |&row| {
                    row.iter()
                        .skip(rect.offset.0)
                        .take(rect.size.0)
                        .all(|&cell| cell == 1)
                })
                .all(|on| on)
        })
        .map(|rect| rect.id)
}

fn main() -> Result<(), Box<dyn Error>> {
    let path = Path::new("inputs/input-03-01.txt");

    let rects = parse_input(path)?;

    let (board, collisions) = part_one(&rects);

    println!("Part 1: {}", collisions);
    println!("Part 2: {:?}", part_two(&rects, board));

    Ok(())
}
