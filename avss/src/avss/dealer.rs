use super::party::AvssParty;
use crate::one2many::prover::One2ManyProver;
use util::algebra::{coset::Coset, field::Field, polynomial::MultilinearPolynomial};
use util::query_result::QueryResult;
use util::random_oracle::RandomOracle;

/// `Dealer` 为 AVSS 协议的 Dealer 部分
pub struct Dealer<T: Field> {
    prover: One2ManyProver<T>,
    evaluations: Vec<MultilinearPolynomial<T>>,
}

impl<T: Field + 'static> Dealer<T> {
    /// `fold` 为折叠函数
    /// `values` 为 `f` 在 `2^i` 次根的集合上的取值，`parameter` 为折叠参数，`coset` 为 `2^i` 次根的集合
    /// 返回 `f` 在 `2^(i+1)` 次根的集合上的取值
    fn fold(values: &Vec<T>, parameter: T, coset: &Coset<T>) -> Vec<T> {
        let len = values.len() / 2;
        let res = (0..len)
            .into_iter()
            .map(|i| {
                let x = values[i];
                let nx = values[i + len];
                let new_v = (x + nx) + parameter * (x - nx) * coset.element_inv_at(i);
                new_v * T::INVERSE_2
            })
            .collect();
        res
    }

    /// `batch_folding` 为批量折叠函数
    /// `total_round` 为协议的总轮数，`polynomial` 为多项式 `f`，`f` 的次数为 `log_d`
    /// `folding_parameter` 为折叠参数，`folding_parameter[i]` 为 `f` 在 `2^i` 次根的集合上的取值
    /// `coset` 为 `2^i` 次根的集合
    /// 返回 `f` 在 `2^i` 次根的集合上的取值
    fn batch_folding(
        total_round: usize,
        polynomial: &MultilinearPolynomial<T>,
        folding_parameter: &Vec<Vec<T>>,
        coset: &Vec<Coset<T>>,
    ) -> (Vec<Vec<Vec<T>>>, Vec<MultilinearPolynomial<T>>) {
        let mut res = vec![vec![(coset[0].fft(polynomial.coefficients().clone()))]];
        let variable_num = polynomial.variable_num();
        let mut evaluations = vec![];
        for round in 0..total_round {
            let len = res[round].len();
            if round < total_round - 1 {
                let mut evaluations = vec![];
                for (index, j) in folding_parameter[round].iter().enumerate() {
                    let next_evaluation =
                        Self::fold(&res[round][index & (len - 1)], *j, &coset[round]);
                    evaluations.push(next_evaluation);
                }
                res.push(evaluations);
            } else {
                for (index, j) in folding_parameter[round].iter().enumerate() {
                    let next_evaluation =
                        Self::fold(&res[round][index & (len - 1)], *j, &coset[round]);
                    let mut coefficients = coset[round + 1].ifft(next_evaluation);
                    coefficients.truncate(1 << (variable_num - total_round));
                    evaluations.push(MultilinearPolynomial::new(coefficients));
                }
            }
        }
        (res, evaluations)
    }

    /// `new` 为 `Dealer` 的构造函数
    /// `total_round` 为协议的总轮数，`polynomial` 为多项式 `f`，`f` 的次数为 `log_d`
    /// `interpolate_coset` 用于插值，`interpolate_cosets[i]` 为 `2^i` 次根的集合
    /// `oracle` 用于生成随机数，返回的随机数的比特长度为 `log_d - terminate_round`
    /// `folding_parameter` 为折叠参数，`folding_parameter[i]` 为 `f` 在 `2^i` 次根的集合上的取值
    /// 返回 `Dealer`
    pub fn new(
        total_round: usize,
        polynomial: &MultilinearPolynomial<T>,
        interpolate_coset: &Vec<Coset<T>>,
        oracle: &RandomOracle<T>,
        folding_parameter: &Vec<Vec<T>>,
    ) -> Self {
        let (functions, evaluations) = Self::batch_folding(
            total_round,
            polynomial,
            folding_parameter,
            interpolate_coset,
        );
        Dealer {
            evaluations,
            prover: One2ManyProver::new(total_round, interpolate_coset, functions, oracle),
        }
    }

    /// `commit_functions` 为 Dealer 向参与方发送 `f` 在 `2^i` 次根的集合上的取值
    /// `avss_party` 为参与方
    pub fn commit_functions(&self, avss_party: &Vec<AvssParty<T>>) {
        let verifiers = avss_party.iter().map(|x| x.verifier.clone()).collect();
        self.prover.commit_functions(&verifiers);
    }

    /// `commit_foldings` 为 Dealer 向参与方发送 `f` 在 `2^(i+1)` 次根的集合上的取值
    /// `avss_party` 为参与方
    pub fn commit_foldings(&self, avss_party: &Vec<AvssParty<T>>) {
        let verifiers = avss_party.iter().map(|x| x.verifier.clone()).collect();
        self.prover.commit_foldings(&verifiers);
    }

    /// `send_evaluations` 为 Dealer 向参与方发送 `f` 在 `2^i` 次根的集合上的取值
    /// `avss_party` 为参与方
    pub fn send_evaluations(&self, avss_party: &mut Vec<AvssParty<T>>) {
        for i in 0..avss_party.len() {
            avss_party[i].set_share(&self.evaluations[i % self.evaluations.len()]);
        }
    }

    /// `prove` 为 Dealer 向参与方发送证明
    pub fn prove(&mut self) {
        self.prover.prove();
    }

    /// `query` 为 Dealer 向参与方发送证明
    pub fn query(&self) -> (Vec<Vec<QueryResult<T>>>, Vec<Vec<QueryResult<T>>>) {
        self.prover.query()
    }
}
