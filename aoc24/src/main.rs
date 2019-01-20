use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader, Error as IoError};
use std::num::ParseIntError;
use std::ops::{Index, IndexMut};
use std::path::Path;
use std::str::FromStr;

use lazy_static::lazy_static;
use regex::Regex;

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
            Error::Invalid(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum Effect {
    Slashing,
    Bludgeoning,
    Fire,
    Cold,
    Radiation,
}

const EFFECTS: usize = 5;

impl FromStr for Effect {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "slashing" => Effect::Slashing,
            "bludgeoning" => Effect::Bludgeoning,
            "fire" => Effect::Fire,
            "cold" => Effect::Cold,
            "radiation" => Effect::Radiation,
            s => return Err(format!("invalid effect: {}", s).into()),
        })
    }
}

#[derive(Copy, Clone, Debug)]
enum Modifier {
    Immune,
    Normal,
    Weak,
}

impl FromStr for Modifier {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "weak" => Modifier::Weak,
            "immune" => Modifier::Immune,
            s => return Err(format!("invalid modifier: {}", s).into()),
        })
    }
}

#[derive(Debug, Clone)]
struct Modifiers([Modifier; EFFECTS]);

impl Modifiers {
    fn new() -> Self {
        Modifiers([Modifier::Normal; EFFECTS])
    }
}

impl Index<Effect> for Modifiers {
    type Output = Modifier;

    fn index(&self, effect: Effect) -> &Self::Output {
        &self.0[effect as usize]
    }
}

impl IndexMut<Effect> for Modifiers {
    fn index_mut(&mut self, effect: Effect) -> &mut Self::Output {
        &mut self.0[effect as usize]
    }
}

#[derive(Debug, Clone)]
struct Group {
    units: u32,
    hit_points: u32,
    modifiers: Modifiers,
    effect: Effect,
    damage: u32,
    initiative: u32,
    order: usize,
}

impl Group {
    fn effective_power(&self) -> u32 {
        self.units * self.damage
    }

    fn potential_damage_to(&self, enemy: &Self) -> u32 {
        self.effective_power() * enemy.modifiers[self.effect] as u32
    }

    fn get_hit_with(&mut self, damage: u32) {
        self.units = self.units.saturating_sub(damage / self.hit_points);
    }

    fn is_alive(&self) -> bool {
        self.units != 0
    }
}

impl FromStr for Group {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(
                r"(?x)
                (?P<units>\d+)\sunits\s
                each\swith\s(?P<hp>\d+)\shit\spoints\s
                (?:\((?P<modifier1>\w+)\sto\s
                (?P<modifiedEffect1>[^);]+)(?:;
                \s(?P<modifier2>\w+)\sto\s
                (?P<modifiedEffect2>[^)]+))?\)\s)?
                with\san\sattack\sthat\sdoes\s
                (?P<damage>\d+)\s(?P<effect>\w+)\sdamage\s
                at\sinitiative\s(?P<initiative>\d+)
            ",
            )
            .unwrap();
        }

        let caps = match RE.captures(s) {
            Some(caps) => caps,
            None => return Err(format!("invalid group pattern: {}", s).into()),
        };

        let units = caps["units"].parse()?;
        let hit_points = caps["hp"].parse()?;
        let effect = caps["effect"].parse()?;
        let damage = caps["damage"].parse()?;
        let initiative = caps["initiative"].parse()?;
        let mut modifiers = Modifiers::new();

        if let Some(effects) = caps.name("modifiedEffect1") {
            let modifier = caps["modifier1"].parse()?;
            for effect in effects.as_str().split(", ") {
                modifiers[effect.parse::<Effect>()?] = modifier;
            }
        }
        if let Some(effects) = caps.name("modifiedEffect2") {
            let modifier = caps["modifier2"].parse()?;
            for effect in effects.as_str().split(", ") {
                modifiers[effect.parse::<Effect>()?] = modifier;
            }
        }

        Ok(Group {
            units,
            hit_points,
            modifiers,
            effect,
            damage,
            initiative,
            order: 0,
        })
    }
}
#[derive(Debug, Clone)]
struct Turn {
    index: usize,
    attacking: Option<usize>,
}

#[derive(Debug, Clone)]
struct Pick {
    index: usize,
    order: (u32, u32),
}

#[derive(Debug, Clone)]
struct Battle {
    groups: Vec<Group>,
    targeted: Vec<bool>,
    separator: usize,
    turn_order: Vec<Turn>,
    immune_system_picking_order: Vec<Pick>,
    infection_picking_order: Vec<Pick>,
}

#[derive(Debug)]
enum EndResult {
    Victory,
    Defeat,
    Deadlock,
}

fn picking_order(groups: &[Group], picking_order: &mut Vec<Pick>, separator: usize) {
    picking_order.clear();
    for (index, group) in groups.iter().enumerate() {
        if group.is_alive() {
            let order = (
                u32::max_value() - group.effective_power(),
                u32::max_value() - group.initiative,
            );
            picking_order.push(Pick {
                index: index + separator,
                order,
            });
        }
    }
    picking_order.sort_unstable_by_key(|&Pick { order, .. }| order);
}

fn attack_order(
    picking_order: &[Pick],
    enemy_picking_order: &[Pick],
    groups: &[Group],
    turn_order: &mut [Turn],
    targeted: &mut [bool],
) {
    for group in picking_order
        .iter()
        .map(|&Pick { index, .. }| &groups[index])
    {
        turn_order[group.order].attacking = if let Some((_, _, _, index)) = enemy_picking_order
            .iter()
            .filter_map(|&Pick { index, .. }| {
                if targeted[index] {
                    None
                } else {
                    let enemy = &groups[index];
                    let potential_damage = group.potential_damage_to(enemy);
                    if potential_damage == 0 {
                        None
                    } else {
                        Some((
                            group.potential_damage_to(enemy),
                            enemy.effective_power(),
                            enemy.initiative,
                            index,
                        ))
                    }
                }
            })
            .max()
        {
            targeted[index] = true;
            Some(index)
        } else {
            None
        }
    }
}

impl Battle {
    fn pick_targets(&mut self) {
        let immune_system_picking_order = &mut self.immune_system_picking_order;
        let infection_picking_order = &mut self.infection_picking_order;
        let separator = self.separator;
        let groups: &[Group] = &self.groups;
        let (immune_system, infection) = groups.split_at(separator);
        let targeted: &mut [bool] = &mut self.targeted;
        for targeted in targeted.iter_mut() {
            *targeted = false;
        }

        picking_order(immune_system, immune_system_picking_order, 0);
        picking_order(infection, infection_picking_order, separator);

        let turn_order: &mut [Turn] = &mut self.turn_order;

        attack_order(
            immune_system_picking_order,
            infection_picking_order,
            groups,
            turn_order,
            targeted,
        );
        attack_order(
            infection_picking_order,
            immune_system_picking_order,
            groups,
            turn_order,
            targeted,
        );
    }

    fn simulate_round(&mut self) {
        for turn in &mut self.turn_order {
            if let Some(index) = turn.attacking {
                let group = &self.groups[turn.index];
                let enemy = &self.groups[index];
                if group.is_alive() {
                    let damage = group.potential_damage_to(enemy);
                    self.groups[index].get_hit_with(damage);
                }
                turn.attacking = None;
            }
        }
    }

    fn remaining_units(&self) -> u32 {
        self.groups.iter().map(|group| group.units).sum()
    }

    fn simulate(&mut self) -> u32 {
        loop {
            self.pick_targets();
            if self.immune_system_picking_order.is_empty()
                || self.infection_picking_order.is_empty()
            {
                return self.remaining_units();
            }
            self.simulate_round();
        }
    }

    fn boost_immune_system(&mut self, boost: u32) {
        for group in &mut self.groups[0..self.separator] {
            group.damage += boost;
        }
    }

    fn simulate_with_boost(&mut self, boost: u32) -> (u32, EndResult) {
        self.boost_immune_system(boost);
        let mut remaining_units = u32::max_value();

        loop {
            self.pick_targets();
            match (
                self.immune_system_picking_order.is_empty(),
                self.infection_picking_order.is_empty(),
                self.remaining_units(),
            ) {
                (false, false, result) => {
                    if result == remaining_units {
                        return (result, EndResult::Deadlock);
                    } else {
                        remaining_units = result;
                    }
                }
                (true, false, result) => return (result, EndResult::Defeat),
                (false, true, result) => return (result, EndResult::Victory),
                _ => panic!(),
            }
            self.simulate_round();
        }
    }

    fn find_smallest_boost(&self) -> (u32, u32) {
        for boost in 0.. {
            if let (remaining_units, EndResult::Victory) = self.clone().simulate_with_boost(boost) {
                return (remaining_units, boost);
            }
        }
        panic!();
    }
}

fn parse_input(path: &Path) -> Result<Battle, Error> {
    let mut lines = BufReader::new(File::open(path)?).lines();
    let mut groups = vec![];
    let mut initiative = vec![];

    match lines
        .next()
        .ok_or_else(|| "unexpected EOF".into())
        .and_then(|result| result.map_err(Error::from))
    {
        Ok(s) => match s.as_ref() {
            "Immune System:" => (),
            _ => return Err(format!(r#"expected "Immune System:", got: {}"#, s).into()),
        },
        Err(e) => return Err(e),
    }

    for (i, line) in (&mut lines).enumerate() {
        let s = line?;
        if s.is_empty() {
            break;
        } else {
            let group: Group = s.parse()?;
            initiative.push((group.initiative, i));
            groups.push(group);
        }
    }

    let separator = groups.len();

    match lines
        .next()
        .ok_or_else(|| "unexpected EOF".into())
        .and_then(|result| result.map_err(Error::from))
    {
        Ok(s) => match s.as_ref() {
            "Infection:" => (),
            _ => return Err(format!(r#"expected "Infection:", got: {}"#, s).into()),
        },
        Err(e) => return Err(e),
    }

    for (i, line) in (separator..).zip(lines) {
        let group: Group = line?.parse()?;
        initiative.push((group.initiative, i));
        groups.push(group);
    }

    initiative.sort_unstable_by_key(|&(initiative, _)| u32::max_value() - initiative);
    let mut turn_order = vec![];

    for (i, (_, index)) in initiative.into_iter().enumerate() {
        turn_order.push(Turn {
            index,
            attacking: None,
        });
        groups[index].order = i;
    }

    let targeted = vec![false; groups.len()];

    Ok(Battle {
        groups,
        targeted,
        turn_order,
        separator,
        immune_system_picking_order: vec![],
        infection_picking_order: vec![],
    })
}

fn main() -> Result<(), Error> {
    let path = Path::new("inputs/input-24-01.txt");

    let battle = parse_input(path)?;

    println!("Part 1: {:?}", battle.clone().simulate());
    println!("Part 2: {:?}", battle.find_smallest_boost().0);

    Ok(())
}
