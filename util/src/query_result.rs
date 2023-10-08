use crate::algebra::field::{as_bytes_vec, Field};
use crate::merkle_tree::MerkleTreeVerifier;
use std::collections::HashMap;
use std::mem::size_of;

#[derive(Clone)]
pub struct QueryResult<T: Field> {
    pub proof_bytes: Vec<u8>,
    pub proof_values: HashMap<usize, T>,
}

impl<T: Field> QueryResult<T> {
    /// `verify_merkle_tree` 用于验证默克尔树
    /// 参数 `leaf_indices` 为叶子节点的索引，`merkle_verifier` 为对应的默克尔树的验证器
    pub fn verify_merkle_tree(
        &self,
        leaf_indices: &Vec<usize>,
        merkle_verifier: &MerkleTreeVerifier,
    ) -> bool {
        let leaves: Vec<Vec<u8>> = leaf_indices
            .iter()
            .map(|x| {
                as_bytes_vec(&[
                    self.proof_values.get(x).unwrap().clone(),
                    self.proof_values
                        .get(&(x + merkle_verifier.leave_number))
                        .unwrap()
                        .clone(),
                ])
            })
            .collect();
        let res = merkle_verifier.verify(self.proof_bytes.clone(), leaf_indices, &leaves);
        assert!(res);
        res
    }

    pub fn proof_size(&self) -> usize {
        self.proof_bytes.len() + self.proof_values.len() * size_of::<T>()
    }
}
