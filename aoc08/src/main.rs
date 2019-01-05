use std::fmt;
use std::fs::File;
use std::io::{prelude::*, BufReader, Error as IoError};
use std::num::ParseIntError;
use std::path::Path;
use std::string::FromUtf8Error;

#[derive(Debug)]
struct Node {
    children: Vec<Node>,
    metadata: Vec<u8>,
}

#[derive(Debug)]
enum Error {
    Io(IoError),
    ParseInt(ParseIntError),
    FromUtf8(FromUtf8Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Io(e) => fmt::Display::fmt(e, f),
            Error::ParseInt(e) => fmt::Display::fmt(e, f),
            Error::FromUtf8(e) => fmt::Display::fmt(e, f),
        }
    }
}

fn populate(vec: &[u8]) -> (Node, usize) {
    let (child_len, meta_len) = (vec[0] as usize, vec[1] as usize);
    let mut children = Vec::with_capacity(child_len);
    let mut metadata = vec![0; meta_len];
    let mut len = 2;

    for _ in 0..child_len {
        let (child, l) = populate(&vec[len..]);

        children.push(child);
        len += l;
    }

    let total = len + meta_len;
    metadata.copy_from_slice(&vec[len..total]);

    (Node { children, metadata }, total)
}

fn parse_input(path: &Path) -> Result<Node, Error> {
    let vec: Result<Vec<_>, _> = BufReader::new(File::open(path).map_err(Error::Io)?)
        .split(b' ')
        .map(|result| {
            result
                .map_err(Error::Io)
                .and_then(|vec| String::from_utf8(vec).map_err(Error::FromUtf8))
                .and_then(|s| s.trim().parse().map_err(Error::ParseInt))
        })
        .collect();

    let (node, _) = populate(&vec?);

    Ok(node)
}

fn metadata_sum(node: &Node) -> u32 {
    node.metadata
        .iter()
        .cloned()
        .fold(0, |sum, x| sum + u32::from(x))
}

fn simple_sum(node: &Node) -> u32 {
    metadata_sum(node) + node.children.iter().map(simple_sum).sum::<u32>()
}

fn complex_sum(node: &Node) -> u32 {
    match node.children.len() {
        0 => metadata_sum(node),
        l => node
            .metadata
            .iter()
            .cloned()
            .filter_map(|x| {
                let x = x as usize;

                if x > 0 && x <= l {
                    Some(&node.children[x - 1])
                } else {
                    None
                }
            })
            .map(complex_sum)
            .sum(),
    }
}

fn main() -> Result<(), Error> {
    let path = Path::new("inputs/input-08-01.txt");

    let root = parse_input(path)?;

    println!("Part 1: {}", simple_sum(&root));
    println!("Part 2: {}", complex_sum(&root));

    Ok(())
}
