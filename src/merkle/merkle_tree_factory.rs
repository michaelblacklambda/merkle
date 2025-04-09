use std::{clone, rc::Rc};

use itertools::Itertools;
use sha256::{digest, try_digest};

use super::merkle_tree::MerkleTree;

#[derive(Clone)]
pub struct MerkleLeaf {
    hash: String,
    data: Vec<u8>,
}

pub struct MerkleTreeFactory {
    leafs: Vec<Rc<MerkleLeaf>>,
}

impl MerkleTreeFactory {
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

        MerkleTreeFactory { leafs: leafs }
    }

    pub fn insert(&mut self, data: Vec<u8>) -> &mut Self {
        let leaf = Rc::new(MerkleLeaf {
            hash: sha256::digest(&data),
            data: data,
        });

        // Tree is no longer valid:
        self.leafs.push(leaf);
        self
    }

    fn create_nodes(nodes: Vec<Rc<MerkleTree>>) -> Vec<Rc<MerkleTree>> {
        /* If odd, pop and add to next tree. */
        if nodes.len() == 1 {
            return nodes;
        }

        let mut pairs = nodes.chunks_exact(2);
        let mut new_nodes = pairs
            .by_ref()
            .map(|pair| {
                let hash = sha256::digest(pair[0].hash.clone() + &pair[1].hash);
                Rc::new(MerkleTree {
                    hash: hash,
                    // I think I need to do something so that I'm not continuously cloning these deep pairs
                    children: Some([Rc::clone(&pair[0]), Rc::clone(&pair[1])]),
                })
            })
            .collect_vec();

        if let [remainder, ..] = pairs.remainder() {
            new_nodes.push(Rc::clone(remainder))
        }

        return new_nodes;
    }

    pub fn create_tree(self) -> Rc<MerkleTree> {
        // verify evenness:
        let mut initial_nodes = self
            .leafs
            .iter()
            .map(|x| {
                Rc::new(MerkleTree {
                    hash: x.hash.clone(),
                    children: None,
                })
            })
            .collect_vec();

        if initial_nodes.len() % 2 != 0 {
            initial_nodes.push(
                initial_nodes
                    .last()
                    .expect("Vec being empty should be impossible")
                    .clone(),
            );
        }

        let root = match Self::create_nodes(initial_nodes).first() {
            Some(merkle_tree) => Rc::clone(merkle_tree),
            None => Rc::new(MerkleTree {
                hash: String::new(),
                children: None,
            }),
        };

        root
    }
}
