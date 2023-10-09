pub mod algebra {
    pub mod coset;
    pub mod field;
    pub mod polynomial;
}
pub mod merkle_tree;
pub mod query_result;
pub mod random_oracle;
pub mod vec_check;

pub const CODE_RATE: usize = 3;
pub const SECURITY_BITS: usize = 100;

/// 将 `n` 分成若干个 2 的幂次，返回这些幂次的集合
pub fn split_n(mut n: usize) -> Vec<usize> {
    let mut res = vec![];
    let mut i = 1;
    while i < n {
        res.push(i);
        n -= i;
        i <<= 1;
    }
    if n > 0 {
        res.push(n);
    }
    res.sort_by(|x, y| y.trailing_zeros().cmp(&x.trailing_zeros()));
    res
}
