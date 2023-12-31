use util::algebra::polynomial::{MultilinearPolynomial, Polynomial};
use util::merkle_tree::MERKLE_ROOT_SIZE;
use util::random_oracle::RandomOracle;
use util::{
    algebra::{coset::Coset, field::Field},
    merkle_tree::MerkleTreeVerifier,
    query_result::QueryResult,
};

#[derive(Clone)]                            // 编译器会自动实现 Clone trait
pub struct One2ManyVerifier<T: Field> {
    total_round: usize,                     // 协议的总轮数
    log_max_degree: usize,                  // 多项式的最大次数的对数
    interpolate_cosets: Vec<Coset<T>>,      // 插值用的 coset，表示插值的余项集合
    function_root: Vec<MerkleTreeVerifier>, // 一个默克尔树的验证器，用于验证多项式的根
    folding_root: Vec<MerkleTreeVerifier>,  // 一个默克尔树的验证器，用于验证折叠的根
    oracle: RandomOracle<T>,                // 一个随机数生成器
    final_value: Option<Polynomial<T>>,     // 多项式的最终值，Option<T> 表示一个可能存在的值，如果存在则为 Some(T)，否则为 None
}

impl<T: Field> One2ManyVerifier<T> {
    pub fn new_with_default_map(
        total_round: usize,
        log_max_degree: usize,
        coset: &Vec<Coset<T>>,
        oracle: &RandomOracle<T>,
    ) -> Self {
        One2ManyVerifier {
            total_round,
            log_max_degree,
            interpolate_cosets: coset.clone(),
            function_root: vec![],
            folding_root: vec![],
            oracle: oracle.clone(),
            final_value: None,
        }
    }

    pub fn new(
        total_round: usize,
        log_max_degree: usize,
        coset: &Vec<Coset<T>>,
        oracle: &RandomOracle<T>,
    ) -> Self {
        One2ManyVerifier {
            total_round,
            log_max_degree,
            interpolate_cosets: coset.clone(),
            function_root: vec![],
            folding_root: vec![],
            oracle: oracle.clone(),
            final_value: None,
        }
    }

    /// `set_function` 用于设置多项式的根
    /// `leave_number` 为默克尔树的叶子节点的数量，`function_root` 存储了MERKLE_ROOT_SIZE长度的字节数组，为默克尔树的根的哈希值。
    /// 向成员变量 `function_root` 中添加一个默克尔树的验证器，用于验证多项式的根。
    pub fn set_function(&mut self, leave_number: usize, function_root: &[u8; MERKLE_ROOT_SIZE]) {
        self.function_root.push(MerkleTreeVerifier {
            merkle_root: function_root.clone(),
            leave_number,
        });
    }

    pub fn receive_folding_root(&mut self, leave_number: usize, folding_root: [u8; MERKLE_ROOT_SIZE]) {
        self.folding_root.push(MerkleTreeVerifier {
            leave_number,
            merkle_root: folding_root,
        });
    }

    pub fn set_final_value(&mut self, value: &Polynomial<T>) {
        assert!(value.degree() <= 1 << (self.log_max_degree - self.total_round));
        self.final_value = Some(value.clone());
    }

    pub fn verify_with_extra_folding(
        &self,
        folding_proofs: &Vec<QueryResult<T>>,
        function_proofs: &Vec<QueryResult<T>>,
        extra_folding_param: &Vec<T>,
        extra_final_poly: &MultilinearPolynomial<T>,
    ) -> bool {
        let mut leaf_indices = self.oracle.query_list.clone();
        for i in 0..self.total_round {
            let domain_size = self.interpolate_cosets[i].size();
            leaf_indices = leaf_indices
                .iter_mut()
                .map(|v| *v % (domain_size >> 1))
                .collect();
            leaf_indices.sort();
            leaf_indices.dedup();

            if i == 0 {
                function_proofs[i].verify_merkle_tree(&leaf_indices, &self.function_root[0]);
            } else {
                folding_proofs[i - 1].verify_merkle_tree(&leaf_indices, &self.folding_root[i - 1]);
            }

            let challenge = self.oracle.folding_challenges[i];
            let get_folding_value = if i == 0 {
                &function_proofs[i].proof_values
            } else {
                &folding_proofs[i - 1].proof_values
            };

            let function_values = if i != 0 {
                let function_query_result = &function_proofs[i];
                function_query_result.verify_merkle_tree(&leaf_indices, &self.function_root[i]);
                Some(&function_query_result.proof_values)
            } else {
                None
            };
            for j in &leaf_indices {
                let x = get_folding_value[j];
                let nx = get_folding_value[&(j + domain_size / 2)];
                let v =
                    x + nx + challenge * (x - nx) * self.interpolate_cosets[i].element_inv_at(*j);
                if i != 0 {
                    let x = function_values.as_ref().unwrap()[j];
                    let nx = function_values.as_ref().unwrap()[&(j + domain_size / 2)];
                    let v = (v * challenge + (x + nx)) * challenge
                        + (x - nx) * self.interpolate_cosets[i].element_inv_at(*j);
                    if i == self.total_round - 1 {
                        let x = self.interpolate_cosets[i + 1].element_at(*j);
                        if v != self.final_value.as_ref().unwrap().evaluation_at(x) {
                            return false;
                        }
                    } else if v != folding_proofs[i].proof_values[j] {
                        return false;
                    }
                } else if folding_proofs.len() > 0 {
                    if v != folding_proofs[i].proof_values[j] {
                        return false;
                    }
                }
                let x = function_proofs[i].proof_values[j];
                let nx = function_proofs[i].proof_values[&(j + domain_size / 2)];
                let v = x
                    + nx
                    + extra_folding_param[i]
                        * (x - nx)
                        * self.interpolate_cosets[i].element_inv_at(*j);
                if i < self.total_round - 1 {
                    assert_eq!(v, function_proofs[i + 1].proof_values[j] * T::from_int(2));
                } else {
                    let x = self.interpolate_cosets[i + 1].element_at(*j);
                    let poly_v = extra_final_poly.evaluate_as_polynomial(x);
                    assert_eq!(v, poly_v * T::from_int(2));
                }
            }
        }
        true
    }
}
