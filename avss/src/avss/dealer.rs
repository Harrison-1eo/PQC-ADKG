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
    /// 根据折叠参数和余元，对输入的值进行折叠操作，生成下一轮的值。
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

    /// `batch_folding` 为批量折叠函数，生成多轮的折叠值。
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

    /// `new` 创建一个 Dealer 实例，初始化协议参数，并使用折叠操作生成协议所需的多项式评估值。
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

   /// 提交多项式评估值的根哈希值给参与方的验证器。 
    pub fn commit_functions(&self, avss_party: &Vec<AvssParty<T>>) {
        // 将 `avss_party` 中的每个元素的 `verifier` 放入 `verifiers`
        let verifiers = avss_party.iter().map(|x| x.verifier.clone()).collect();
        self.prover.commit_functions(&verifiers);
    }

    /// 将折叠证明提交给参与方的验证器，以供验证。
    pub fn commit_foldings(&self, avss_party: &Vec<AvssParty<T>>) {
        let verifiers = avss_party.iter().map(|x| x.verifier.clone()).collect();
        self.prover.commit_foldings(&verifiers);
    }

    /// 将生成的多项式评估值发送给参与方，以供他们继续执行协议。
    pub fn send_evaluations(&self, avss_party: &mut Vec<AvssParty<T>>) {
        for i in 0..avss_party.len() {
            avss_party[i].set_share(&self.evaluations[i % self.evaluations.len()]);
        }
    }

    /// 根据初始化的协议参数和评估值，执行证明生成过程，生成协议的证明。
    pub fn prove(&mut self) {
        self.prover.prove();
    }

    /// `query` 为 Dealer 向参与方发送协议的证明信息，包括折叠和插值部分的证明信息，以供验证。
    pub fn query(&self) -> (Vec<Vec<QueryResult<T>>>, Vec<Vec<QueryResult<T>>>) {
        self.prover.query()
    }
}

