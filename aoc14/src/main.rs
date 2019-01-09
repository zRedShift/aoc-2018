const INPUT: usize = 320_851;
const INPUT_ARRAY: [u8; 6] = [3, 2, 0, 8, 5, 1];

fn part_one() {
    let mut recipes: Vec<u8> = vec![3, 7];
    let mut first = 0;
    let mut second = 1;

    while recipes.len() < INPUT + 10 {
        let sum = recipes[first] + recipes[second];

        if sum < 10 {
            recipes.push(sum);
        } else {
            recipes.push(1);
            recipes.push(sum - 10);
        }

        first = (first + (recipes[first] + 1) as usize) % recipes.len();
        second = (second + (recipes[second] + 1) as usize) % recipes.len();
    }

    print!("Part 1: ");
    for x in recipes.iter().skip(INPUT).take(10).cloned() {
        print!("{}", x);
    }
    println!();
}

fn compare(recipes: &[u8]) -> bool {
    INPUT_ARRAY == recipes[recipes.len() - INPUT_ARRAY.len()..]
}

fn part_two() {
    let mut recipes: Vec<u8> = vec![3, 7, 1, 0, 1, 0, 1, 2, 4, 5];
    let mut first = 6;
    let mut second = 3;

    loop {
        let sum = recipes[first] + recipes[second];
        if sum < 10 {
            recipes.push(sum);

            if compare(&recipes) {
                break;
            }
        } else {
            recipes.push(1);
            if compare(&recipes) {
                break;
            }

            recipes.push(sum - 10);
            if compare(&recipes) {
                break;
            }
        }

        first = (first + (recipes[first] + 1) as usize) % recipes.len();
        second = (second + (recipes[second] + 1) as usize) % recipes.len();
    }

    println!("Part 2: {}", recipes.len() - INPUT_ARRAY.len());
}

fn main() {
    part_one();
    part_two();
}
