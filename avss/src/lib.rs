pub mod one2many {
    pub mod prover;
    pub mod verifier;
}
pub mod avss {
    pub mod dealer;
    pub mod party;
}

use avss::dealer::Dealer;
use avss::party::AvssParty;
use util::algebra::field::Field;
use util::algebra::polynomial::MultilinearPolynomial;
use util::random_oracle::RandomOracle;

use util::algebra::coset::Coset;
use util::algebra::field::mersenne61_ext::Mersenne61Ext;
use util::split_n;
use util::{CODE_RATE, SECURITY_BITS};


pub fn avss_deal(log_n: usize, terminate_round: usize) {
    
    let log_t = log_n - 2;          // 余元集合的大小
    let log_d = log_t * 2;          // 多项式的次数（变量数）

    // oracle 用于生成随机数
    let oracle = RandomOracle::new(log_d - terminate_round, SECURITY_BITS / CODE_RATE);
    
    // interpolate_cosets 为余元集合，第一个余元是随机生成的，后面的余元是前一个余元的平方
    let mut interpolate_cosets = vec![Coset::new(
        1 << (log_t * 2 + CODE_RATE),
        Mersenne61Ext::random_element(),
    )];
    for i in 1..log_d {
        interpolate_cosets.push(interpolate_cosets[i - 1].pow(2));
    }

    // polynomial 为多项式
    let polynomial = MultilinearPolynomial::random_polynomial(log_d);

    //生成变换参数 coset_x 和 coset_y，使用这些参数创建两个余元集合 coset_x 和 coset_y
    let x_shift = Mersenne61Ext::random_element();
    let coset_x = Coset::new(1 << log_n, x_shift);
    let y_shift = Mersenne61Ext::random_element();
    let coset_y = Coset::new(1 << log_n, y_shift);

    // folding_parameter 存储多轮的折叠参数
    let mut folding_parameter = vec![];
    let v = split_n((1 << log_t) - 1);
    for i in &v {
        folding_parameter.push(coset_x.pow(*i).all_elements());
    }
    let last_len = folding_parameter.last().unwrap().len();
    for i in &v {
        folding_parameter.push(
            coset_y
                .pow(*i)
                .all_elements()
                .iter()
                .map(|x| (0..last_len).into_iter().map(|_| *x).collect::<Vec<_>>())
                .flatten()
                .collect(),
        );
    }

    let mut parties = vec![];
    for i in 0..(1 << (log_n * 2)) {
        let mut open_point = vec![];
        for j in 0..log_d {
            open_point.push(folding_parameter[j][i % folding_parameter[j].len()]);
        }
        parties.push(AvssParty::new(
            log_d - terminate_round,
            &interpolate_cosets,
            open_point,
            &oracle,
        ));
    }

    let mut dealer = Dealer::new(
        log_d - terminate_round,
        &polynomial,
        &interpolate_cosets,
        &oracle,
        &folding_parameter,
    );

    // 为每个参与方分配秘密份额
    dealer.send_evaluations(&mut parties);
    // 向每个参与方发送整个多项式的承诺
    dealer.commit_functions(&parties);
    // 执行协议的证明过程
    dealer.prove();
    // 向每个参与方发送折叠多项式的承诺
    dealer.commit_foldings(&parties);
    // 执行协议的证明过程，向参与方发送协议的证明信息，包括折叠和插值部分的证明信息，以供验证。
    dealer.query();
}



pub fn avss_verify(log_n: usize, terminate_round: usize) {
    let log_t = log_n - 2;
    let log_d = log_t * 2;
    let oracle = RandomOracle::new(log_d - terminate_round, SECURITY_BITS / CODE_RATE);
    let mut interpolate_cosets = vec![Coset::new(
        1 << (log_t * 2 + CODE_RATE),
        Mersenne61Ext::random_element(),
    )];
    for i in 1..log_d {
        interpolate_cosets.push(interpolate_cosets[i - 1].pow(2));
    }
    let polynomial = MultilinearPolynomial::random_polynomial(log_d);

    let x_shift = Mersenne61Ext::random_element();
    let coset_x = Coset::new(1 << log_n, x_shift);
    let mut folding_parameter = vec![];
    let v = split_n((1 << log_t) - 1);
    for i in &v {
        folding_parameter.push(coset_x.pow(*i).all_elements());
    }
    let y_shift = Mersenne61Ext::random_element();
    let coset_y = Coset::new(1 << log_n, y_shift);
    let last_len = folding_parameter.last().unwrap().len();
    for i in &v {
        folding_parameter.push(
            coset_y
                .pow(*i)
                .all_elements()
                .iter()
                .map(|x| (0..last_len).into_iter().map(|_| *x).collect::<Vec<_>>())
                .flatten()
                .collect(),
        );
    }
    let mut parties = vec![];
    for i in 0..(1 << (log_n * 2)) {
        let mut open_point = vec![];
        for j in 0..log_d {
            open_point.push(folding_parameter[j][i % folding_parameter[j].len()]);
        }
        parties.push(AvssParty::new(
            log_d - terminate_round,
            &interpolate_cosets,
            open_point,
            &oracle,
        ));
    }
    let mut dealer = Dealer::new(
        log_d - terminate_round,
        &polynomial,
        &interpolate_cosets,
        &oracle,
        &folding_parameter,
    );
    dealer.send_evaluations(&mut parties);
    dealer.commit_functions(&parties);
    dealer.prove();
    dealer.commit_foldings(&parties);
    let (folding, function) = dealer.query();
    let mut folding0 = vec![];
    let mut function0 = vec![];
    for i in 0..(log_d - terminate_round) {
        if i < log_d - terminate_round - 1 {
            folding0.push(folding[i][0].clone());
        }
        function0.push(function[i][0].clone());
    }
    
    assert!(parties[0].verify(&folding0, &function0));
}
