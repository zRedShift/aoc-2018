use std::fmt;
use std::fs::File;
use std::io::{Error as IoError, Read};
use std::ops::{Index, IndexMut};
use std::path::Path;
use std::str::FromStr;

use self::Action::*;
use self::Direction::*;
use self::Entity::*;
use self::Target::*;

const UNREACHABLE: u8 = u8::max_value();
const DIRECTIONS: [Direction; 4] = [North, West, East, South];
const HP: HitPoints = HitPoints(200);
const AP: u8 = 3;

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
            Error::Invalid(s) => write!(f, "invalid input: {}", s),
        }
    }
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
struct HitPoints(u8);

impl HitPoints {
    fn hit(&mut self) -> bool {
        match self.0.overflowing_sub(AP) {
            (0, _) | (_, true) => {
                self.0 = 0;
                true
            }
            (new, _) => {
                self.0 = new;
                false
            }
        }
    }
}

#[derive(Debug)]
enum Entity {
    Empty,
    Wall,
    Elf(HitPoints),
    Goblin(HitPoints),
}

impl fmt::Display for Entity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Empty => write!(f, "Empty Space"),
            Wall => write!(f, "Wall"),
            Elf(hp) => write!(f, "Elf with {} hit points", hp.0),
            Goblin(hp) => write!(f, "Goblin with {} hit points", hp.0),
        }
    }
}

impl Entity {
    fn die(&mut self) {
        *self = Empty;
    }
}

enum Direction {
    North,
    West,
    East,
    South,
}

enum Target {
    Found(Position),
    NotFound(u32, bool),
    Unreachable,
}
#[derive(Debug)]
enum Action {
    Wait,
    Attack(Position),
    Move,
}

#[derive(Debug, Copy, Clone)]
struct Position(usize);

impl Position {
    fn to(self, width: usize, direction: &Direction) -> Option<Self> {
        match direction {
            North => self.0.checked_sub(width).map(Position),
            West if self.0 % width != 0 => self.0.checked_sub(1).map(Position),
            East if self.0 % width != width - 1 => self.0.checked_add(1).map(Position),
            South => self.0.checked_add(width).map(Position),
            _ => None,
        }
    }
}

#[derive(Debug)]
struct Board {
    entities: Vec<Entity>,
    width: usize,
}

impl FromStr for Board {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = s.as_bytes();

        let width = match bytes.iter().enumerate().find(|&(_, &x)| x == b'\n') {
            Some((i, _)) => i,
            None => bytes.len(),
        };

        match bytes.iter().find_map(|&x| match x {
            b'.' | b'#' | b'G' | b'E' | b'\n' => None,
            x => Some(x),
        }) {
            None => Ok(()),
            Some(x) => Err(format!("invalid character: {}", char::from(x),)),
        }?;

        let mut entities = Vec::with_capacity(bytes.len());

        entities.extend(bytes.iter().filter_map(|&x| match x {
            b'.' => Some(Empty),
            b'#' => Some(Wall),
            b'G' => Some(Goblin(HP)),
            b'E' => Some(Elf(HP)),
            _ => None,
        }));

        Ok(Board { entities, width })
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for y in self.entities.chunks_exact(self.width) {
            for x in y.iter() {
                match x {
                    Goblin(_) => write!(f, "G"),
                    Elf(_) => write!(f, "E"),
                    Wall => write!(f, "#"),
                    Empty => write!(f, "."),
                }?;
            }

            writeln!(f)?;
        }

        Ok(())
    }
}

impl Index<Position> for [u8] {
    type Output = u8;

    fn index(&self, index: Position) -> &Self::Output {
        &self[index.0]
    }
}

impl IndexMut<Position> for [u8] {
    fn index_mut(&mut self, index: Position) -> &mut Self::Output {
        &mut self[index.0]
    }
}

impl Index<Position> for Vec<Entity> {
    type Output = Entity;

    fn index(&self, index: Position) -> &Self::Output {
        &self[index.0]
    }
}

impl IndexMut<Position> for Vec<Entity> {
    fn index_mut(&mut self, index: Position) -> &mut Self::Output {
        &mut self[index.0]
    }
}

impl Board {
    fn calculate_turn_order(&mut self, turn_order: &mut Vec<Position>) {
        turn_order.clear();

        turn_order.extend(self.entities.iter().enumerate().filter_map(
            |(i, entity)| match entity {
                Elf(_) | Goblin(_) => Some(Position(i)),
                _ => None,
            },
        ));
    }

    fn move_to(&mut self, old: Position, new: Position) {
        self.entities.swap(old.0, new.0);
    }

    fn attack(&mut self, position: Position) {
        let victim = &mut self.entities[position];

        match victim {
            Goblin(hp) | Elf(hp) => {
                if hp.hit() {
                    victim.die()
                }
            }
            entity => panic!("attempting to attack {}", entity,),
        }
    }

    fn pick_action(&self, position: Position) -> Action {
        let elf = match self.entities[position] {
            Elf(_) => true,
            Goblin(_) => false,
            // target died before the end of turn
            _ => return Wait,
        };

        let mut can_move = false;

        let target = DIRECTIONS
            .iter()
            .filter_map(|direction| {
                match (
                    position
                        .to(self.width, direction)
                        .map(|position| &self.entities[position]),
                    elf,
                ) {
                    (Some(Goblin(hp)), true) | (Some(Elf(hp)), false) => Some((hp, position)),
                    (Some(Empty), _) => {
                        can_move = true;
                        None
                    }
                    _ => None,
                }
            })
            .min_by_key(|(hp, _)| hp.0)
            .map(|(_, position)| position);

        match (target, can_move) {
            (Some(position), _) => Attack(position),
            (None, true) => Move,
            _ => Wait,
        }
    }

    fn calculate_path(&self, position: Position, distance: u8, pathfinding: &mut [u8]) {
        pathfinding[position] = distance;

        for direction in DIRECTIONS.iter() {
            match position
                .to(self.width, direction)
                .map(|position| (position, &self.entities[position], pathfinding[position]))
            {
                Some((position, Empty, new_dist)) if new_dist > distance + 1 => {
                    self.calculate_path(position, distance + 1, pathfinding)
                }
                _ => (),
            }
        }
    }

    fn update_paths(&self, position: Position, pathfinding: &mut [u8]) {
        for x in pathfinding.iter_mut() {
            *x = UNREACHABLE;
        }
        self.calculate_path(position, 0, pathfinding);
    }

    fn remaining_hp(&self) -> u32 {
        self.entities
            .iter()
            .filter_map(|entity| match entity {
                Goblin(hp) | Elf(hp) => Some(u32::from(hp.0)),
                _ => None,
            })
            .sum()
    }

    fn find_closest_target(&self, position: Position, pathfinding: &[u8]) -> Target {
        let elf = match &self.entities[position] {
            Elf(_) => true,
            Goblin(_) => false,
            entity => panic!("invalid entity {} for finding a target", entity),
        };

        match self
            .entities
            .iter()
            .enumerate()
            .filter_map(|(i, entity)| match (entity, elf) {
                (Elf(_), false) | (Goblin(_), true) => Some(Position(i)),
                _ => None,
            })
            .flat_map(|position| {
                DIRECTIONS.iter().filter_map(move |direction| {
                    if let Some((position, Empty)) = position
                        .to(self.width, direction)
                        .map(|position| (position, &self.entities[position]))
                    {
                        Some((position, pathfinding[position]))
                    } else {
                        None
                    }
                })
            })
            .min_by_key(|(_, distance)| *distance)
        {
            Some((_, UNREACHABLE)) => Unreachable,
            Some((position, _)) => Found(position),
            None => NotFound(self.remaining_hp(), elf),
        }
    }

    fn find_path_to_target(&self, target: Position, pathfinding: &[u8]) -> Position {
        match DIRECTIONS
            .iter()
            .filter_map(|direction| {
                target
                    .to(self.width, direction)
                    .map(|position| (position, pathfinding[position]))
            })
            .min_by_key(|(_, distance)| *distance)
            .unwrap()
        {
            (_, 0) => target,
            (position, _) => self.find_path_to_target(position, pathfinding),
        }
    }

    fn simulate(&mut self) -> (u32, u32, bool) {
        let turn_order = &mut Vec::new();
        let pathfinding = &mut vec![0; self.entities.len()];
        let mut round = 0;

        let (hp, elf_victory) = 'outer: loop {
            self.calculate_turn_order(turn_order);

            println!("Round {}:", round);
            for &position in turn_order.iter() {
                println!("{}", self.entities[position]);
            }
            println!("{}", self);

            for &position in turn_order.iter() {
                match self.pick_action(position) {
                    Wait => println!("{} waits.", self.entities[position]),
                    Attack(target) => {
                        println!(
                            "{} attacks {}.",
                            self.entities[position], self.entities[target]
                        );
                        self.attack(target)
                    }
                    Move => {
                        self.update_paths(position, pathfinding);
                        match self.find_closest_target(position, pathfinding) {
                            Found(target) => {
                                println!(
                                    "{} moves.",
                                    self.entities[position]
                                );
                                self.move_to(
                                    position,
                                    self.find_path_to_target(target, pathfinding),
                                )
                            }
                            Unreachable => println!("{} waits.", self.entities[position]),
                            NotFound(hp, elf_victory) => break 'outer (hp, elf_victory),
                        }
                    }
                }
            }

            round += 1;
        };

        (round, hp, elf_victory)
    }
}

fn parse_input(path: &Path) -> Result<Board, Error> {
    let mut s = String::new();
    File::open(path)?.read_to_string(&mut s)?;

    Ok(s.parse()?)
}

fn main() -> Result<(), Error> {
    let path = Path::new("inputs/input-15-00.txt");

    let mut board = parse_input(path)?;

    println!("{:?}", board.simulate());

    Ok(())
}
