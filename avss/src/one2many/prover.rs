use std::{cell::RefCell, rc::Rc};

use super::verifier::One2ManyVerifier;
use util::algebra::polynomial::Polynomial;

use util::merkle_tree::MERKLE_ROOT_SIZE;
use util::query_result::QueryResult;
use util::{
    algebra::{
        coset::Coset,
        field::{as_bytes_vec, Field},
    },
    merkle_tree::MerkleTreeProver,
    random_oracle::RandomOracle,
};


/// 插值值，包含插值值和Merkle树
struct InterpolateValue<T: Field> {
    value: Vec<T>,
    merkle_tree: MerkleTreeProver,
}

impl<T: Field> InterpolateValue<T> {
    /// 创建一个 InterpolateValue 实例，其中包含了多项式在余元集合上的取值，以及对应的 Merkle 树。
    /// value中的前半部分为多项式在余元集合上的取值，后半部分为多项式在余元集合上的取值的逆变换。
    fn new(value: Vec<T>) -> Self {
        let len = value.len() / 2;
        let merkle_tree = MerkleTreeProver::new(
            (0..len)
                .map(|i| as_bytes_vec(&[value[i], value[i + len]]))
                .collect(),
        );
        Self { value, merkle_tree }
    }

    /// 多项式在余元集合上的取值的个数，实际上为 value 的长度的一半。
    fn leave_num(&self) -> usize {
        self.merkle_tree.leave_num()
    }

    /// 返回 Merkle 树根的哈希值
    fn commit(&self) -> [u8; MERKLE_ROOT_SIZE] {
        self.merkle_tree.commit()
    }

    /// 查询指定索引的证明信息，包括证明的哈希值和证明的取值。
    /// 返回值中的 proof_bytes 为需要证明的叶子结点的证明路径，proof_values 为证明的取值。
    fn query(&self, leaf_indices: &Vec<usize>) -> QueryResult<T> {
        let len = self.merkle_tree.leave_num();
        let proof_values = leaf_indices
            .iter()
            // .map(|j| [(*j, self.value[*j]), (*j + len, self.value[*j + len])])
            .flat_map(|j| [(*j, self.value[*j]), (*j + len, self.value[*j + len])])
            .collect();
        let proof_bytes = self.merkle_tree.open(&leaf_indices);
        QueryResult {
            proof_bytes,
            proof_values,
        }
    }
}


/// 插值集合，是InterpolateValue的Vector
struct CosetInterpolate<T: Field> {
    interpolates: Vec<InterpolateValue<T>>,
}

impl<T: Field> CosetInterpolate<T> {
    fn len(&self) -> usize {
        self.interpolates.len()
    }

    fn new(functions: Vec<Vec<T>>) -> Self {
        CosetInterpolate {
            interpolates: functions
                .into_iter()
                .map(|values| InterpolateValue::new(values))
                .collect(),
        }
    }

    /// 返回插值集合中第一个 InterpolateValue 的 leave_num
    fn field_size(&self) -> usize {
        self.interpolates[0].value.len()
    }

    fn from_interpolates(interpolates: Vec<InterpolateValue<T>>) -> Self {
        CosetInterpolate { interpolates }
    }

    /// 从集合中获取指定索引的`InterpolateValue`实例
    fn get_interpolation(&self, index: usize) -> &InterpolateValue<T> {
        let len = self.interpolates.len();
        assert!((len & (len - 1)) == 0);
        &self.interpolates[index & (len - 1)]       // 由于len为2的幂，因此index & (len - 1) 等价于 index % len
    }
}

/// 一个证明者对多个验证者进行证明
pub struct One2ManyProver<T: Field> {
    total_round: usize,
    interpolate_cosets: Vec<Coset<T>>,
    functions: Vec<CosetInterpolate<T>>,
    foldings: Vec<CosetInterpolate<T>>,
    oracle: RandomOracle<T>,
    final_value: Vec<Polynomial<T>>,
}

impl<T: Field> One2ManyProver<T> {
    
    pub fn new(
        total_round: usize,
        interpolate_coset: &Vec<Coset<T>>,
        functions: Vec<Vec<Vec<T>>>,
        oracle: &RandomOracle<T>,
    ) -> One2ManyProver<T> {
        assert_eq!(total_round, functions.len());
        // functions: Vec<CosetInterpolate<T>>，是参数中functions的每个元素转换成CosetInterpolate<T>的结果
        // 每个CosetInterpolate中包含了多个InterpolateValue
        // 每个InterpolateValue包含了多项式在余元集合上的取值，以及对应的Merkle树
        let functions: Vec<CosetInterpolate<T>> = functions
            .into_iter()
            .map(|x| CosetInterpolate::new(x))
            .collect();

        One2ManyProver {
            total_round,
            interpolate_cosets: interpolate_coset.clone(),
            functions,
            foldings: vec![],
            oracle: oracle.clone(),
            final_value: vec![],
        }
    }

    /// 函数需要 total_round * len(Verifiers) 个 InterpolateValue，其中每个InterpolateValue包含了多项式在余元集合上的取值，以及对应的Merkle树。
    /// 向verifiers中的每个验证者的`function_root` 中添加total_round个默克尔树的验证器，用于验证多项式的根。
    pub fn commit_functions(&self, verifiers: &Vec<Rc<RefCell<One2ManyVerifier<T>>>>) {
        for i in 0..self.total_round {
            for (idx, j) in verifiers.into_iter().enumerate() {
                let function = self.functions[i].get_interpolation(idx);
                j.borrow_mut()
                    .set_function(function.leave_num(), &function.commit());
            }
        }
    }

    /// 前 total_round - 1 轮，向每个验证者的 `folding_root` 中添加 total_round - 1 个默克尔树的验证器，用于验证折叠的根。
    /// 最后一轮，向每个验证者的 `final_value` 中添加一个多项式，用于验证最终值。
    /// 函数中需要self.foldings中有(total_round - 1) * len(verifiers)个InterpolateValue
    pub fn commit_foldings(&self, verifiers: &Vec<Rc<RefCell<One2ManyVerifier<T>>>>) {
        for i in 0..(self.total_round - 1) {
            for (idx, j) in verifiers.into_iter().enumerate() {
                let interpolation = self.foldings[i].get_interpolation(idx);
                j.borrow_mut()
                    .receive_folding_root(interpolation.leave_num(), interpolation.commit());
            }
        }
        for i in 0..verifiers.len() {
            verifiers[i]
                .borrow_mut()
                .set_final_value(&self.final_value[i % self.final_value.len()]);
        }
    }

    /// 根据当前轮数、滚动函数索引和挑战值，计算下一轮的评估值。
    /// res 的长度为 len / 2 
    fn evaluation_next_domain(
        &self,
        round: usize,
        rolling_function_index: usize,
        challenge: T,
    ) -> Vec<T> {
        let mut res = vec![];
        let len = self.functions[round].field_size();
        let get_folding_value = if round == 0 {
            self.functions[round].get_interpolation(rolling_function_index)
        } else {
            self.foldings[round - 1].get_interpolation(rolling_function_index)
        };
        let coset = &self.interpolate_cosets[round];
        for i in 0..(len / 2) {
            let x = get_folding_value.value[i];
            let nx = get_folding_value.value[i + len / 2];
            let new_v = (x + nx) + challenge * (x - nx) * coset.element_inv_at(i);
            if round == 0 {
                res.push(new_v);
            } else {
                let fv = &self.functions[round].interpolates[rolling_function_index];
                let x = fv.value[i];
                let nx = fv.value[i + len / 2];
                let new_v =
                    (new_v * challenge + (x + nx)) * challenge + (x - nx) * coset.element_inv_at(i);
                res.push(new_v);
            }
        }
        res
    }

    /// 根据给定的证明协议参数和数据，生成证明的各个部分，包括插值、折叠和最终值。
    pub fn prove(&mut self) {
        for i in 0..self.total_round {
            let challenge = self.oracle.folding_challenges[i];
            if i < self.total_round - 1 {
                let mut interpolates = vec![];
                // 对于每个需要进行插值的函数，计算插值值，并将其添加到 interpolates 中
                for j in 0..self.functions[i].len() {
                    let next_evalutation = self.evaluation_next_domain(i, j, challenge);
                    let interpolate_value = InterpolateValue::new(next_evalutation);
                    interpolates.push(interpolate_value);
                }
                self.foldings
                    .push(CosetInterpolate::from_interpolates(interpolates));
            } else {
                // 对于每个需要折叠的函数，计算折叠值，并将其添加到 interpolates 中
                for j in 0..self.functions[i].len() {
                    let next_evalutation = self.evaluation_next_domain(i, j, challenge);
                    let coefficients = self.interpolate_cosets[i + 1].ifft(next_evalutation);
                    self.final_value.push(Polynomial::new(coefficients));
                }
            }
        }
    }

    /// 查询证明中的信息，包括插值和折叠部分的证明信息，以便供验证器验证。
    pub fn query(&self) -> (Vec<Vec<QueryResult<T>>>, Vec<Vec<QueryResult<T>>>) {
        let mut folding_res = vec![];
        let mut functions_res = vec![];
        let mut leaf_indices = self.oracle.query_list.clone();

        for i in 0..self.total_round {
            let len = self.functions[i].field_size();
            leaf_indices = leaf_indices.iter_mut().map(|v| *v % (len >> 1)).collect();
            leaf_indices.sort();
            leaf_indices.dedup();

            if i == 0 {
                let query_result = self.functions[0].get_interpolation(0).query(&leaf_indices);
                functions_res.push(vec![query_result]);
            } else {
                let query_result = self.functions[i]
                    .interpolates
                    .iter()
                    .map(|x| x.query(&leaf_indices))
                    .collect();
                functions_res.push(query_result);
            }

            if i > 0 {
                folding_res.push(
                    self.foldings[i - 1]
                        .interpolates
                        .iter()
                        .map(|x| x.query(&leaf_indices))
                        .collect(),
                );
            }
        }
        (folding_res, functions_res)
    }
}
