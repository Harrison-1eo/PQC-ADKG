use crate::one2many::verifier::One2ManyVerifier;
use std::{cell::RefCell, rc::Rc};
use util::algebra::{coset::Coset, field::Field, polynomial::MultilinearPolynomial};
use util::query_result::QueryResult;
use util::random_oracle::RandomOracle;

#[derive(Clone)]
/// `AvssParty` 为 AVSS 协议的参与方部分
pub struct AvssParty<T: Field> {
    pub verifier: Rc<RefCell<One2ManyVerifier<T>>>,
    open_point: Vec<T>,
    final_poly: Option<MultilinearPolynomial<T>>,
}

impl<T: Field + 'static> AvssParty<T> {
    /// `share` 为参与方的秘密份额
    pub fn share(&self) -> T {
        let poly = self.final_poly.as_ref().unwrap();
        let variable_num = poly.variable_num();
        let n = self.open_point.len();
        poly.evaluate(&self.open_point[n - variable_num..].to_vec())
    }

    pub fn interpolate_share(&self) -> Vec<T> {
        let poly = self.final_poly.as_ref().unwrap();
        let variable_num = poly.variable_num();
        let n = self.open_point.len();
        let opn = self.open_point[n - variable_num..].to_vec();
        let eva = poly.evaluate(&opn);
        vec![opn[0], eva]
    }

    pub fn all_share(&self) -> MultilinearPolynomial<T>  {
        self.final_poly.as_ref().unwrap().clone()
    }

    /// `set_share` 为设置参与方的秘密份额
    /// `final_poly` 为多项式 `f`，`f` 的次数为 `log_d`
    pub fn set_share(&mut self, final_poly: &MultilinearPolynomial<T>) {
        self.final_poly = Some(final_poly.clone());
    }

    /// `open_point` 为参与方的开点
    pub fn open_point(&self) -> &Vec<T> {
        &self.open_point
    }

    /// `new` 为构造函数
    /// `total_round` 为协议的总轮数，`interpolate_coset` 为插值所用的 `2^i` 次根的集合
    /// `open_point` 为参与方的开点，`oracle` 用于生成随机数
    pub fn new(
        total_round: usize,
        interpolate_coset: &Vec<Coset<T>>,
        open_point: Vec<T>,
        oracle: &RandomOracle<T>,
    ) -> AvssParty<T> {
        AvssParty {
            verifier: Rc::new(RefCell::new(One2ManyVerifier::new_with_default_map(
                total_round,
                open_point.len(),
                interpolate_coset,
                oracle,
            ))),
            open_point,
            final_poly: None,
        }
    }

    /// `verify` 为验证函数
    /// `folding_proofs` 为折叠证明，`function_proofs` 为函数证明
    /// 返回验证结果
    pub fn verify(
        &self,
        folding_proofs: &Vec<QueryResult<T>>,
        function_proofs: &Vec<QueryResult<T>>,
    ) -> bool {
        self.verifier.borrow().verify_with_extra_folding(
            &folding_proofs,
            &function_proofs,
            &self.open_point,
            self.final_poly.as_ref().unwrap(),
        )
    }
}
