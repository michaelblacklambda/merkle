use std::{clone, rc::Rc};

use itertools::Itertools;
use sha256::{digest, try_digest};

#[derive(Clone)]
enum MerkleType {
    Leaf(Rc<MerkleLeaf>),
    Node(Rc<MerkleNode>),
}

impl MerkleType {
    pub fn hash(&self) -> String {
        match self {
            MerkleType::Leaf(merkle_leaf) => merkle_leaf.hash.clone(),
            MerkleType::Node(merkle_node) => merkle_node.hash.clone(),
        }
    }
}

enum Direction {
    LEFT,
    RIGHT,
}

#[derive(Clone)]
pub struct MerkleLeafs {
    leafs: Vec<Rc<MerkleLeafs>>,
}

#[derive(Clone)]
pub struct MerkleNode {
    hash: String,
    children: [Rc<MerkleType>; 2],
}

#[derive(Clone)]

pub struct MerkleLeaf {
    hash: String,
    data: Vec<u8>,
}

pub struct MerkleTree {
    root: Option<Rc<MerkleType>>,
    leafs: Vec<Rc<MerkleLeaf>>,
}

impl MerkleLeaf {
    fn hash(left_node: MerkleLeaf, right_node: MerkleLeaf) -> String {
        let s = left_node.hash + &right_node.hash;
        sha256::digest(s)
    }
}

impl MerkleTree {
    pub fn new(data: Vec<Vec<u8>>) -> Self {
        let leafs = data
            .into_iter()
            .map(|d| {
                Rc::new(MerkleLeaf {
                    hash: sha256::digest(&d),
                    data: d,
                })
            })
            .collect_vec();

        MerkleTree {
            root: None,
            leafs: leafs,
        }
    }

    pub fn insert(&mut self, data: Vec<u8>) -> &mut Self {
        let leaf = Rc::new(MerkleLeaf {
            hash: sha256::digest(&data),
            data: data,
        });

        // Tree is no longer valid:
        self.root = None;
        self.leafs.push(leaf);
        self
    }

    fn create_nodes(nodes: Vec<Rc<MerkleType>>) -> Vec<Rc<MerkleType>> {
        /* If odd, pop and add to next tree. */
        if nodes.len() == 1 {
            return nodes;
        }

        let mut pairs = nodes.chunks_exact(2);
        let mut new_nodes = pairs
            .by_ref()
            .map(|pair| {
                let hash = sha256::digest(pair[0].hash() + &pair[1].hash());
                Rc::new(MerkleType::Node(Rc::new(MerkleNode {
                    hash: hash,
                    // I think I need to do something so that I'm not continuously cloning these deep pairs
                    children: [Rc::clone(&pair[0]), Rc::clone(&pair[1])],
                })))
            })
            .collect_vec();

        if let [remainder, ..] = pairs.remainder() {
            new_nodes.push(Rc::clone(remainder))
        }

        return new_nodes;
    }

    pub fn create_tree(&mut self) {
        // verify evenness:
        let leafs = if self.leafs.len() % 2 == 0 {
            &self.leafs
        } else {
            self.leafs.push(self.leafs.last().unwrap().clone());
            &self.leafs
        };

        let root = Self::create_nodes(
            leafs
                .iter()
                .map(|x| Rc::new(MerkleType::Leaf(Rc::clone(x))))
                .collect_vec(),
        );

        self.root = root.first().map(Rc::clone);
    }
}
