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

use rand::Rng; 

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
    
    // interpolate_cosets 首先生成一个生成元，阶为 2 ^ (log_t * 2 + CODE_RATE)
    let mut interpolate_cosets = vec![Coset::new(
        1 << (log_t * 2 + CODE_RATE),
        Mersenne61Ext::random_element(),
    )];
    // 然后生成 log_d - 1 个余元，每个余元是前一个余元的平方
    for i in 1..log_d {
        interpolate_cosets.push(interpolate_cosets[i - 1].pow(2));
    }

    // 生成一个随机多项式，次数为 2 ^ log_d，即 n^2/16
    let polynomial = MultilinearPolynomial::random_polynomial(log_d);

    // 生成两个生成元 coset_x 和 coset_y，阶为 2 ^ log_n，即 n
    // 使用这些参数创建两个余元集合 coset_x 和 coset_y
    let x_shift = Mersenne61Ext::random_element();
    let coset_x = Coset::new(1 << log_n, x_shift);
    let y_shift = Mersenne61Ext::random_element();
    let coset_y = Coset::new(1 << log_n, y_shift);

    // folding_parameter 存储多轮的折叠参数
    // 首先得到 t - 1 的二进制表达的数组，比如 t = 4，那么 v = [1, 2]
    // 然后计算出 coset_x 的 v 次方，得到一个二维数组，每一行都是 coset_x 的 v[i] 次方的所有元素
    let mut folding_parameter = vec![];
    let v = split_n((1 << log_t) - 1);
    for i in &v {
        folding_parameter.push(coset_x.pow(*i).all_elements());
    }
    
    // 计算最后一个元素的长度
    // 遍历 v 中的每一个元素，计算 coset_y 的 v[i] 次方，得到一个二维数组，每一行都是 coset_y 的 v[i] 次方的所有元素
    // 第二层map函数使得每个元素都重复 last_len 次
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

    // 输出折叠参数
    for i in 0..folding_parameter.len() {
        println!("{}: {:?}", i, folding_parameter[i]);
    }

    // parties 存储参与方，有 n^2 个参与方
    let mut parties = vec![];
    for i in 0..(1 << (log_n * 2)) {
        let mut open_point = vec![];
        // 每个参与方有 log_d 个开放点，每个开放点都是折叠参数的一个元素
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

    let mut shares = vec![];
    for i in 0..(1 << (log_n * 2)) {
        assert_eq!(parties[i].share(), polynomial.evaluate(&parties[i].open_point()));
        println!("{}: open point: {:?}, share: [{}]", i, parties[i].interpolate_share().0, parties[i].interpolate_share().1);
        shares.push(vec!(parties[i].interpolate_share().0[0], parties[i].interpolate_share().1));
    }

    // let a = 1 << log_d + 1;

    // for i in 0..4 {
    //     // 随机选取 a 个参与方
    //     let mut selected_parties = vec![];
    //     while true {
    //         let s = rand::thread_rng().gen_range(0.. 1 << (log_n * 2));
    //         if !selected_parties.contains(&s) {
    //             selected_parties.push(s);
    //         }
    //         if selected_parties.len() == a {
    //             break;
    //         }
    //     }
    //     // 选取这 a 个参与方的秘密份额
    //     let mut selected_shares = vec![];
    //     for j in &selected_parties {
    //         selected_shares.push(shares[*j].clone());
    //     }
    //     // 插值
    //     let res = MultilinearPolynomial::interpolate(&selected_shares, &interpolate_cosets);
    //     // 计算多项式在 0 处的值
    //     let res0 = res.evaluate(&vec![Mersenne61Ext::from_int(0); log_d]);
    //     println!("{}: {}", i, res0);
    // }

    for i in 0..(1 << (log_n * 2)) {
        // println!("{}: {}, {}", i, parties[i].all_share().evaluate(&vec![Mersenne61Ext::from_int(0); log_d]), 
        //                           parties[i].all_share().evaluate_as_polynomial(Mersenne61Ext::from_int(0)));
        println!("{}: {}", i, parties[i].all_share().evaluate_as_polynomial(Mersenne61Ext::from_int(0)));
    }
}
