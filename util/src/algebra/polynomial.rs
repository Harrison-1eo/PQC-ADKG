use super::coset::Coset;
use super::field::Field;

#[derive(Debug, Clone)]
pub struct Polynomial<T: Field> {
    coefficients: Vec<T>,
}

impl<T: Field> Polynomial<T> {
    pub fn new(mut coefficients: Vec<T>) -> Polynomial<T> {
        let zero = T::from_int(0);
        while *coefficients.last().unwrap() == zero {
            coefficients.pop();
        }
        Polynomial { coefficients }
    }

    pub fn coefficients(&self) -> &Vec<T> {
        &self.coefficients
    }

    pub fn random_polynomial(degree: usize) -> Polynomial<T> {
        Polynomial {
            coefficients: (0..degree).map(|_| Field::random_element()).collect(),
        }
    }

    pub fn degree(&self) -> usize {
        let n = self.coefficients.len();
        if n == 0 {
            0
        } else {
            n - 1
        }
    }

    pub fn evaluation_at(&self, x: T) -> T {
        let mut res = Field::from_int(0);
        for i in self.coefficients.iter().rev() {
            res *= x;
            res += *i;
        }
        res
    }

    pub fn evaluation_over_coset(&self, coset: &Coset<T>) -> Vec<T> {
        coset.fft(self.coefficients.clone())
    }

    pub fn over_vanish_polynomial(
        &self,
        vanishing_polynomial: &VanishingPolynomial<T>,
    ) -> Polynomial<T> {
        let degree = vanishing_polynomial.degree;
        let low_term = vanishing_polynomial.shift;
        let mut coeff = vec![];
        let mut remnant = self.coefficients.clone();
        for i in (degree..self.coefficients.len()).rev() {
            let tmp = remnant[i] * low_term;
            coeff.push(remnant[i]);
            remnant[i - degree] += tmp;
        }
        coeff.reverse();
        Polynomial::new(coeff)
    }
}

#[derive(Debug, Clone)]
pub struct VanishingPolynomial<T: Field> {
    degree: usize,
    shift: T,
}

impl<T: Field> VanishingPolynomial<T> {
    pub fn new(coset: &Coset<T>) -> VanishingPolynomial<T> {
        let degree = coset.size();
        VanishingPolynomial {
            degree,
            shift: coset.shift().pow(degree),
        }
    }

    pub fn evaluation_at(&self, x: T) -> T {
        x.pow(self.degree) - self.shift
    }
}

#[derive(Debug, Clone)]
pub struct MultilinearPolynomial<T: Field> {
    coefficients: Vec<T>,
}

impl<T: Field> MultilinearPolynomial<T> {
    /// 返回多项式的系数的集合的引用
    pub fn coefficients(&self) -> &Vec<T> {
        &self.coefficients
    }

    pub fn new(coefficients: Vec<T>) -> Self {
        let len = coefficients.len();
        assert_eq!(len & (len - 1), 0);
        MultilinearPolynomial { coefficients }
    }

    pub fn folding(&self, parameter: T) -> Self {
        let coefficients = Self::folding_vector(&self.coefficients, parameter);
        MultilinearPolynomial { coefficients }
    }

    fn folding_vector(v: &Vec<T>, parameter: T) -> Vec<T> {
        let len = v.len();
        assert_eq!(len & (len - 1), 0);
        let mut res = vec![];
        for i in (0..v.len()).step_by(2) {
            res.push(v[i] + parameter * v[i + 1]);
        }
        res
    }

    /// 创建一个随机的多项式，其中包含 2 ^ variable_num 个变量
    pub fn random_polynomial(variable_num: usize) -> Self {
        MultilinearPolynomial {
            coefficients: (0..(1 << variable_num))
                .map(|_| Field::random_element())
                .collect(),
        }
    }

    /// 计算多项式在给定点的值
    pub fn evaluate(&self, point: &Vec<T>) -> T {
        let len = self.coefficients.len();
        assert_eq!(1 << point.len(), self.coefficients.len());
        let mut res = self.coefficients.clone();
        for (index, coeff) in point.iter().enumerate() {
            for i in (0..len).step_by(2 << index) {
                let x = *coeff * res[i + (1 << index)];
                res[i] += x;
            }
        }
        res[0]
    }

    pub fn evaluate_as_polynomial(&self, point: T) -> T {
        let mut res = Field::from_int(0);
        for i in self.coefficients.iter().rev() {
            res *= point;
            res += *i;
        }
        res
    }

    /// 返回多项式的变量的个数的log2
    pub fn variable_num(&self) -> usize {
        self.coefficients.len().ilog2() as usize
    }
}

impl<T: Field> MultilinearPolynomial<T> {

    /// 插值多项式
    pub fn interpolate(
        evaluations: &Vec<Vec<T>>,
        interpolate_coset: &Vec<Coset<T>>,
    ) -> MultilinearPolynomial<T> {
        let mut res = evaluations[0].clone();
        let mut tmp = vec![];
        for i in 1..evaluations.len() {
            for j in 0..res.len() {
                tmp.push(res[j] - evaluations[i][j]);
            }
            res = tmp;
            tmp = vec![];
        }
        let mut res = interpolate_coset[0].ifft(res);
        res.truncate(1 << interpolate_coset[0].size().ilog2());
        MultilinearPolynomial::new(res)
    }
}

#[cfg(test)]
mod test {
    use crate::algebra::field::mersenne61_ext::Mersenne61Ext;

    use super::super::field::fp64::Fp64;
    use super::*;

    #[test]
    fn evaluation() {
        let coset = Coset::new(32, Fp64::random_element());
        let all_elements = coset.all_elements();
        let poly = Polynomial::random_polynomial(32);
        let eval = poly.evaluation_over_coset(&coset);
        for i in 0..coset.size() {
            assert_eq!(eval[i], poly.evaluation_at(all_elements[i]));
        }
        let poly = VanishingPolynomial::new(&coset);
        for i in coset.all_elements() {
            assert_eq!(Fp64::from_int(0), poly.evaluation_at(i));
        }
    }

    #[test]
    fn multilinear() {
        let poly = MultilinearPolynomial::random_polynomial(8);
        let point = (0..8).map(|_| Mersenne61Ext::random_element()).collect();
        let v = poly.evaluate(&point);

        println!("{:?} \n {:?} \n {:?}", poly, point, v);

        let mut folding_poly = poly.clone();
        for parameter in point {
            folding_poly = folding_poly.folding(parameter);
        }
        assert_eq!(folding_poly.coefficients.len(), 1);
        assert_eq!(folding_poly.coefficients[0], v);

        let z = Mersenne61Ext::random_element();
        let beta = Mersenne61Ext::random_element();
        let folding_poly = poly.folding(z);
        let a = poly.evaluate_as_polynomial(beta);
        let b = poly.evaluate_as_polynomial(-beta);
        let c = folding_poly.evaluate_as_polynomial(beta * beta);
        let v = a + b + z * (a - b) * beta.inverse();
        assert_eq!(v * Mersenne61Ext::from_int(2).inverse(), c);
    }
}
