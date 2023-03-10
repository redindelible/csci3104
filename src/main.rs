#![feature(portable_simd)]

mod pool_compute;

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{Read, Write};
use std::ops::Range;
use std::time::Instant;

// use std::simd::u64x4;
// type SIMDType = u64x4;
// const SIMD_SIZE: usize = 256;
// const LANE_SIZE: usize = 64;
// const SIMD_COUNT: usize = 3;

// use std::simd::u64x8;
// type SIMDType = u64x8;
// const SIMD_SIZE: usize = 512;
// const LANE_SIZE: usize = 64;
// const SIMD_COUNT: usize = 2;

use std::simd::u8x32;
use crate::pool_compute::pool_compute;

type SIMDType = u8x32;
const SIMD_SIZE: usize = 256;
const LANE_SIZE: usize = 8;
const SIMD_COUNT: usize = 3;

// use std::simd::u8x16;
// type SIMDType = u8x16;
// const SIMD_SIZE: usize = 128;
// const LANE_SIZE: usize = 8;
// const SIMD_COUNT: usize = 6;

struct Set {
    items: [SIMDType; SIMD_COUNT],
    count: usize
}

impl Set {
    fn new() -> Set {
        Set {
            items: Default::default(), count: 0
        }
    }

    fn mark(&mut self, idx: u16) {
        let which = (idx as usize)/ SIMD_SIZE;
        let idx = (idx as usize) % SIMD_SIZE;
        self.items[which].as_mut_array()[(idx as usize) / LANE_SIZE] |= 1 << ((idx as usize) % LANE_SIZE);
        self.count += 1;
    }

    fn is_subset(&self, other: &Set) -> bool {
        (0..SIMD_COUNT).all(|i| self.items[i] & other.items[i] == self.items[i])
    }
}

struct Node {
    s: Set,
    name: String
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Node {
    fn new(name: String, s: Set) -> Node {
        Node {
            name, s
        }
    }

    fn is_subset(&self, other: &Node) -> bool {
        self.s.is_subset(&other.s)
    }

    fn count(&self) -> usize {
        self.s.count
    }
}

#[allow(unused)]
fn render(link: (&Node, &Node)) -> String {
    return format!("{}->{}", link.0.name, link.1.name)
}

#[allow(unused)]
fn verify(links: &Vec<(&Node, &Node)>, sol_path: &str, show: bool) {
    let mut failed = false;
    let mut sol = String::new();
    File::open(sol_path).unwrap().read_to_string(&mut sol).unwrap();

    let mut sol_links = HashSet::new();
    for line in sol.lines() {
        sol_links.insert(String::from(line));
    }

    for &link in links {
        let rendered = render(link);
        if !sol_links.contains(&rendered) {
            failed = true;
            if show {
                println!("Extraneous link: {}", rendered);
            }
        } else {
            sol_links.remove(&rendered);
        }
    }

    if sol_links.len() != 0 {
        for item in &sol_links {
            failed = true;
            if show {
                println!("Missing Link: {}", item)
            }
        }
    }
    if !failed {
        println!("Verified Successfully!")
    } else {
        println!("Verification Failed.")
    }
}

#[allow(unused)]
fn display(links: &Vec<(&Node, &Node)>) {
    let mut f = File::create("correct_sol.txt").unwrap();

    for &link in links {
        f.write_all(format!("{}\n", render(link)).as_bytes()).unwrap();
    }
}

struct GraphData {
    nodes: Vec<Vec<Node>>
}

impl GraphData {
    fn new() -> GraphData {
        GraphData {
            nodes: Vec::with_capacity(20)
        }
    }
}

struct Graph<'a> {
    data: &'a mut GraphData,
    level_count: usize
}

impl<'a> Graph<'a> {
    fn new(data: &'a mut GraphData) -> Self {
        data.nodes.clear();
        data.nodes.reserve(20);
        Graph {
            data, level_count: 0
        }
    }
    fn insert_node(&mut self, node: Node) {
        if node.count() >= self.level_count {
            self.data.nodes.extend((self.level_count..=node.count()).map(|_| Vec::new()));
            self.level_count = node.count()+1
        }
        self.data.nodes[node.count()].push(node)
    }

    fn levels_iter(&self) -> Range<usize> {
        0..self.level_count
    }

    fn levels_above(&self, level: usize) -> Range<usize> {
        level+1..self.level_count
    }

    fn level(&self, level: usize) -> &Vec<Node> {
        &self.data.nodes[level]
    }
}

#[allow(unused)]
fn construct_and_verify(name: &str, prob: &str, sol: &str) {
    println!("-----------------");
    println!("Running {}", name);
    println!("-----------------");
    let start = Instant::now();
    let curr = Instant::now();

    let mut contents = String::new();
    File::open(prob).unwrap().read_to_string(&mut contents).unwrap();

    let mut int_map: HashMap<u16, u16> = HashMap::new();

    let mut data = GraphData::new();
    let mut graph = Graph::new(&mut data);

    for line in contents.lines() {
        let mut set = Set::new();
        for num in line.split_whitespace() {
            let siz = int_map.len() as u16;
            let idx = *int_map.entry(num.parse().unwrap()).or_insert(siz);
            set.mark(idx);
        }

        graph.insert_node(Node::new(line.replace(" ", ", "),set));
    }
    if int_map.len() > SIMD_SIZE * SIMD_COUNT {
        panic!("Bit Set not large enough ({} bits needed)", int_map.len());
    }

    println!(" +  Constructing sets took {} sec", curr.elapsed().as_secs_f32());
    let curr = Instant::now();

    let mut links: Vec<(&Node, &Node)> = Vec::new();

    for count in graph.levels_iter().rev() {
        if count+1 >= graph.level_count {
            continue
        }

        let this_links = pool_compute(graph.level(count).iter(), 20, |node: &Node| {
            let mut linked: Vec<&Node> = Vec::new();
            let mut ret = Vec::new();
            for c in graph.levels_above(count) {
                let node_group = graph.level(c);
                for other_node in node_group {
                    if node.is_subset(&other_node) {
                        if !linked.iter().any(|&linked_node| linked_node.is_subset(&other_node)) {
                            ret.push((node, other_node));
                            linked.push(other_node)
                        }
                    }
                }
            };
            ret
        });

        links.extend(this_links.iter().flatten());
    }

    println!(" +  Constructing links took {} sec", curr.elapsed().as_secs_f32());
    println!(" +  Total Algorithm took {} sec", start.elapsed().as_secs_f32());
    let curr = Instant::now();

    verify(&links, sol, true);
    println!(" +  Verification took {} sec", curr.elapsed().as_secs_f32());
    // display(&links);
}

fn main() {
    // construct_and_verify("4 items", "ex1.txt", "ex1_sol.txt");
    // construct_and_verify("Example", "ex2.txt", "ex2_sol.txt");
    construct_and_verify("Long", "79867.txt", "correct_sol.txt");
}
