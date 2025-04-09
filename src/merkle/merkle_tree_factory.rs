use std::{clone, rc::Rc};

use itertools::Itertools;
use sha256::{digest, try_digest};

use super::merkle_tree::MerkleTree;

#[derive(Clone, Debug)]
pub struct MerkleLeaf {
    hash: String,
    data: Vec<u8>,
}

#[derive(Clone, Debug)]
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

        MerkleTreeFactory::create_nodes(new_nodes)
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

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use itertools::Itertools;

    use crate::merkle::merkle_tree::MerkleTree;

    use super::MerkleTreeFactory;

    #[test]
    fn test_find_element() {
        let data = vec!["1", "2", "3", "4", "5", "6", "7", "8"]
            .into_iter()
            .map(|x| x.as_bytes().to_vec())
            .collect_vec();

        let factory = MerkleTreeFactory::new(data.clone());
        let tree = factory.create_tree();
        println!("Tree: {:#?}", tree);
        let hash = sha256::digest("7".as_bytes().to_vec());
        // let found = MerkleTree::find(tree, hash);
        let proof = MerkleTree::construct_proof(Rc::clone(&tree), hash).unwrap_or_default();

        let hashes = data
            .clone()
            .into_iter()
            .map(|x| sha256::digest(x))
            .collect_vec();
        // println!("Data: {:#?}", hashes);

        // println!("Found: {:#?}", proof);

        let verified_hash = MerkleTree::verify_proof(proof);

        println!("verified proof: {:#?}", verified_hash);
        assert_eq!(verified_hash, Rc::clone(&tree).hash);
    }
}
