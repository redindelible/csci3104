#![feature(portable_simd)]

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::simd::u64x4;
use std::time::Instant;

const ITEMS: usize = 3;

struct Set {
    items: [u64x4; ITEMS],
    count: usize
}

impl Set {
    fn new() -> Set {
        Set {
            items: Default::default(), count: 0
        }
    }

    fn mark(&mut self, idx: u16) {
        let which = idx / 256;
        let idx = (idx as usize) % 256;
        self.items[which as usize].as_mut_array()[(idx as usize) / 64] |= 1 << ((idx as usize) % 64);
        self.count += 1;
    }

    fn is_subset(&self, other: &Set) -> bool {
        (0..ITEMS).all(|i| self.items[i] & other.items[i] == self.items[i])
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

    fn is_superset(&self, other: &Node) -> bool {
        other.s.is_subset(&other.s)
    }
}

fn render(link: (&Node, &Node)) -> String {
    return format!("{}->{}", link.0.name, link.1.name)
}

fn verify(links: &Vec<(&Node, &Node)>, sol_path: &str) {
    let mut sol = String::new();
    File::open(sol_path).unwrap().read_to_string(&mut sol).unwrap();

    let mut sol_links = HashSet::new();
    for line in sol.lines() {
        sol_links.insert(String::from(line));
    }

    for &link in links {
        let rendered = render(link);
        if !sol_links.contains(&rendered) {
            println!("Extraneous link: {}", rendered);
        } else {
            sol_links.remove(&rendered);
        }
    }

    if sol_links.len() != 0 {
        for item in &sol_links {
            println!("Missing Link: {}", item)
        }
    } else {
        println!("Verified Successfully!")
    }
}

fn display(links: &Vec<(&Node, &Node)>) {
    for &link in links {
        println!("{}", render(link))
    }
}

fn construct_and_verify(name: &str, prob: &str, sol: &str) {
    println!("-----------------");
    println!("Running {}", name);
    println!("-----------------");
    let start = Instant::now();
    let curr = Instant::now();

    let mut contents = String::new();
    File::open(prob).unwrap().read_to_string(&mut contents).unwrap();

    let mut int_map: HashMap<u16, u16> = HashMap::new();

    let mut nodes: Vec<Vec<Node>> = Vec::with_capacity(20);
    let mut max_count: usize = 0;
    nodes.push(Vec::with_capacity(5));

    for line in contents.lines() {
        let mut set = Set::new();
        for num in line.split_whitespace() {
            let siz = int_map.len() as u16;
            let idx = *int_map.entry(num.parse().unwrap()).or_insert(siz);
            set.mark(idx);
        }
        if set.count > max_count  {
            nodes.extend((max_count..=set.count).map(|_| Vec::new()));
            max_count = set.count;
        }

        nodes[set.count as usize].push(Node::new(line.replace(" ", ", "),set));
    }
    println!(" +  Constructing sets took {} sec", curr.elapsed().as_secs_f32());
    let curr = Instant::now();

    let mut links: Vec<(&Node, &Node)> = Vec::new();

    for count in (0..nodes.len()).rev() {
        for node in &nodes[count] {
            let mut linked: Vec<&Node> = Vec::new();
            for c in count+1..=max_count {
                let node_group = &nodes[c];
                for other_node in node_group {
                    if node.is_subset(&other_node) {
                        if !linked.iter().any(|&linked_node| linked_node.is_subset(&other_node)) {
                            links.push((node, other_node));
                            linked.push(other_node)
                        }
                    }
                }
            }
        }
    }

    println!(" +  Constructing links took {} sec", curr.elapsed().as_secs_f32());
    println!(" +  Total Algorithm took {} sec", start.elapsed().as_secs_f32());
    let curr = Instant::now();

    verify(&links, sol);
    println!(" +  Verification took {} sec", curr.elapsed().as_secs_f32());
    // display(&links);
}

fn main() {
    // construct_and_verify("4 items", "ex1.txt", "ex1_sol.txt");
    // construct_and_verify("Example", "ex2.txt", "ex2_sol.txt");
    construct_and_verify("Long", "79867.txt", "long_sol.txt");
    // println!("Size of int map: {}", int_map.len());
}
