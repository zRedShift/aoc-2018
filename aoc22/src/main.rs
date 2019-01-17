const DEPTH: u32 = 8787;
const TARGET_X: usize = 10;
const TARGET_Y: usize = 725;
const GEO_INDEX_X: u32 = 16807;
const GEO_INDEX_Y: u32 = 48271;
const EROSION: u32 = 20183;
const MOD: u32 = 3;
const MAP_Y: usize = TARGET_Y + 50;
const MAP_X: usize = TARGET_X + 20;
const SWITCH_TIME: u16 = 7;
const MOVE_TIME: u16 = 1;

#[derive(Copy, Clone)]
enum Terrain {
    Rocky,
    Wet,
    Narrow,
}

#[derive(Copy, Clone)]
enum Tool {
    Torch,
    ClimbingGear,
    Neither,
}

type Map = [[Terrain; MAP_X]; MAP_Y];
type Times = [[(u16, u16); MAP_X]; MAP_Y];

fn map() -> Map {
    let mut map = [[Terrain::Rocky; MAP_X]; MAP_Y];

    let mut row = [0u32; MAP_X];
    for (x, cell) in row.iter_mut().enumerate() {
        let erosion = (x as u32 * GEO_INDEX_X + DEPTH) % EROSION;
        *cell = erosion;
        map[0][x] = match erosion % MOD {
            1 => Terrain::Wet,
            2 => Terrain::Narrow,
            _ => Terrain::Rocky,
        };
    }

    let mut column = [0u32; MAP_Y];
    for (y, cell) in column.iter_mut().enumerate() {
        let erosion = (y as u32 * GEO_INDEX_Y + DEPTH) % EROSION;
        *cell = erosion;
        map[y][0] = match erosion % MOD {
            1 => Terrain::Wet,
            2 => Terrain::Narrow,
            _ => Terrain::Rocky,
        };
    }

    for (y, &first) in column.iter().enumerate().skip(1) {
        let mut prev = first;

        for (x, cell) in row.iter_mut().enumerate().skip(1) {
            let erosion = (prev * *cell + DEPTH) % EROSION;
            *cell = erosion;
            map[y][x] = match erosion % MOD {
                1 => Terrain::Wet,
                2 => Terrain::Narrow,
                _ => Terrain::Rocky,
            };

            prev = erosion;
        }
    }

    map
}

fn sum(map: &Map) -> u32 {
    map.iter()
        .take(TARGET_Y + 1)
        .flat_map(|row| {
            row.iter()
                .cloned()
                .take(TARGET_X + 1)
                .map(|terrain| terrain as u32)
        })
        .sum()
}

fn move_to(
    map: &Map,
    times: &mut Times,
    x: usize,
    y: usize,
    time: u16,
    tool: Tool,
    terrain: Terrain,
) {
    let (duration, tool) = match (terrain, map[y][x], tool) {
        (Terrain::Rocky, Terrain::Wet, Tool::Torch)
        | (Terrain::Wet, Terrain::Rocky, Tool::Neither) => (SWITCH_TIME, Tool::ClimbingGear),
        (Terrain::Wet, Terrain::Narrow, Tool::ClimbingGear)
        | (Terrain::Narrow, Terrain::Wet, Tool::Torch) => (SWITCH_TIME, Tool::Neither),
        (Terrain::Rocky, Terrain::Narrow, Tool::ClimbingGear)
        | (Terrain::Narrow, Terrain::Rocky, Tool::Neither) => (SWITCH_TIME, Tool::Torch),
        _ => (0, tool),
    };

    calculate_times(map, times, x, y, time + duration + MOVE_TIME, tool);
}

fn calculate_times(map: &Map, times: &mut Times, x: usize, y: usize, time: u16, tool: Tool) {
    let terrain = map[y][x];
    let old_time = match (terrain, tool) {
        (Terrain::Rocky, Tool::Torch)
        | (Terrain::Wet, Tool::ClimbingGear)
        | (Terrain::Narrow, Tool::Torch) => &mut times[y][x].0,
        _ => &mut times[y][x].1,
    };

    if time >= *old_time || time > (TARGET_X + TARGET_Y) as u16 * 2 {
        return;
    }

    *old_time = time;

    if y != 0 {
        move_to(map, times, x, y - 1, time, tool, terrain);
    }
    if x != 0 {
        move_to(map, times, x - 1, y, time, tool, terrain);
    }
    if y != MAP_Y - 1 {
        move_to(map, times, x, y + 1, time, tool, terrain);
    }
    if x != MAP_X - 1 {
        move_to(map, times, x + 1, y, time, tool, terrain);
    }
}

fn fastest_time(map: &Map) -> u16 {
    let mut times: Times = [[(u16::max_value(), u16::max_value()); MAP_X]; MAP_Y];
    let tool = Tool::Torch;

    calculate_times(map, &mut times, 0, 0, 0, tool);

    let times = times[TARGET_Y][TARGET_X];

    if times.0 > times.1 + SWITCH_TIME {
        times.1 + SWITCH_TIME
    } else {
        times.0
    }
}

fn main() {
    let mut map = map();
    map[TARGET_Y][TARGET_X] = Terrain::Rocky;
    println!("Part 1: {}", sum(&map));
    println!("Part 2: {}", fastest_time(&map));
}
