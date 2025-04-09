use std::{cmp::Ordering, rc::Rc};

use itertools::Itertools;

#[derive(Clone, Debug)]
pub struct MerkleTree {
    pub hash: String,
    pub children: Option<[Rc<MerkleTree>; 2]>,
}

#[derive(PartialEq, Eq, Debug)]
pub enum MerkleDirection {
    Left,
    Right,
}

impl MerkleTree {
    pub fn find(tree: Rc<MerkleTree>, hash: String) -> Option<Vec<Rc<MerkleTree>>> {
        if tree.hash == hash {
            return Some(vec![Rc::clone(&tree)]);
        }

        let mut results = match &tree.children {
            Some(children) => MerkleTree::find(Rc::clone(&children[0]), hash.clone())
                .or(MerkleTree::find(Rc::clone(&children[1]), hash))?,
            None => return None,
        };

        results.push(Rc::clone(&tree));

        return Some(results);
    }

    fn get_direction_hash_from_pair(
        child: Rc<MerkleTree>,
        parent: Rc<MerkleTree>,
    ) -> (MerkleDirection, String) {
        let children = match &parent.children {
            Some(c) => c,
            None => panic!("Parents should always have children"),
        };
        if children[0].hash == child.hash {
            (MerkleDirection::Right, children[1].hash.clone())
        } else {
            (MerkleDirection::Left, children[0].hash.clone())
        }
    }

    pub fn construct_proof(
        tree: Rc<MerkleTree>,
        hash: String,
    ) -> Option<Vec<(MerkleDirection, String)>> {
        let proof_nodes = MerkleTree::find(tree, hash)?;
        let mut proof = proof_nodes
            .windows(2)
            .map(|window| {
                MerkleTree::get_direction_hash_from_pair(
                    Rc::clone(&window[0]),
                    Rc::clone(&window[1]),
                )
            })
            .collect_vec();

        // Append the first elements direction
        let matching_node = proof_nodes.first()?;
        let first_element = if proof.first()?.0 == MerkleDirection::Left {
            (MerkleDirection::Right, matching_node.hash.clone())
        } else {
            (MerkleDirection::Left, matching_node.hash.clone())
        };

        proof.insert(0, first_element);
        Some(proof)
    }

    pub fn verify_proof(proof: Vec<(MerkleDirection, String)>) -> String {
        let mut proof_iter = proof.into_iter();
        let mut current_hash = match proof_iter.next() {
            Some(elem) => elem.1,
            None => return String::new(),
        };

        for (direction, hash) in proof_iter {
            current_hash = match direction {
                MerkleDirection::Left => sha256::digest(hash + &current_hash),
                MerkleDirection::Right => sha256::digest(current_hash + &hash),
            }
        }

        return current_hash;
    }
}
