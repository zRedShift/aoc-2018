const SIZE: usize = 300;
const INPUT: usize = 9005;

fn calculate(x: usize, y: usize) -> i8 {
    let rack_id = x + 11;
    (((rack_id * y + rack_id + INPUT) * rack_id) % 1000 / 100) as i8 - 5
}

fn populate(grid: &mut [[i8; SIZE]; SIZE]) {
    for (y, row) in grid.iter_mut().enumerate() {
        for (x, cell) in row.iter_mut().enumerate() {
            *cell = calculate(x, y);
        }
    }
}

fn sum(grid: &[[i8; SIZE]; SIZE], x: usize, y: usize, size: usize) -> i32 {
    grid.iter()
        .skip(y)
        .take(size)
        .flat_map(|y| y.iter().skip(x).take(size).cloned().map(i32::from))
        .sum()
}

fn part_one(grid: &[[i8; SIZE]; SIZE]) -> (usize, usize) {
    let size = 2;

    let (x, y) = (0..SIZE - size)
        .flat_map(|y| (0..SIZE - size).map(move |x| (x, y)))
        .max_by_key(|&(x, y)| sum(grid, x, y, size + 1))
        .unwrap();

    (x + 1, y + 1)
}

fn part_two(grid: &[[i8; SIZE]; SIZE]) -> (usize, usize, usize) {
    let (x, y, size) = (0..SIZE)
        .flat_map(|size| {
            (0..SIZE - size).flat_map(move |y| (0..SIZE - size).map(move |x| (x, y, size)))
        })
        .max_by_key(|&(x, y, size)| sum(grid, x, y, size + 1))
        .unwrap();

    (x + 1, y + 1, size + 1)
}

fn main() {
    let mut grid = [[0i8; SIZE]; SIZE];

    populate(&mut grid);

    println!("Part 1: {:?}", part_one(&grid));
    println!("Part 2: {:?}", part_two(&grid));
}
