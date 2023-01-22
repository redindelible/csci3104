#![feature(portable_simd)]

use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::File;
use std::io::Read;
use std::iter::Map;
use std::ops::Range;
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

    fn count(&self) -> usize {
        self.s.count
    }
}

fn render(link: (&Node, &Node)) -> String {
    return format!("{}->{}", link.0.name, link.1.name)
}

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

fn display(links: &Vec<(&Node, &Node)>) {
    for &link in links {
        println!("{}", render(link))
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
    println!(" +  Constructing sets took {} sec", curr.elapsed().as_secs_f32());
    let curr = Instant::now();

    let mut links: Vec<(&Node, &Node)> = Vec::new();

    for count in graph.levels_iter().rev() {
        if count+1 >= graph.level_count {
            continue
        }

        for node in graph.level(count) {
            let mut linked: Vec<&Node> = Vec::new();
            for c in graph.levels_above(count) {
                let node_group = graph.level(c);
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

    verify(&links, sol, false);
    println!(" +  Verification took {} sec", curr.elapsed().as_secs_f32());
    // display(&links);
}

fn main() {
    // construct_and_verify("4 items", "ex1.txt", "ex1_sol.txt");
    // construct_and_verify("Example", "ex2.txt", "ex2_sol.txt");
    construct_and_verify("Long", "79867.txt", "long_sol.txt");
}
