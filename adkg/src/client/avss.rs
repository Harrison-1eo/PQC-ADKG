use avss::avss::dealer::Dealer;
use avss::avss::party::AvssParty;

use util::algebra::field::Field;
use util::algebra::polynomial::MultilinearPolynomial;
use util::random_oracle::RandomOracle;
use util::algebra::coset::Coset;
use util::algebra::field::mersenne61_ext::Mersenne61Ext;
use util::split_n;
use util::{CODE_RATE, SECURITY_BITS};

use crate::msg::message::Message;
use crate::msg::message::MessageType;

pub struct AvssNode{
    id: usize,
    log_n: usize,
    terminate_round: usize,
    polynomial: MultilinearPolynomial<Mersenne61Ext>,
    dealer: Dealer<Mersenne61Ext>,
    parties: Vec<AvssParty<Mersenne61Ext>>,
}

impl AvssNode {

    pub fn new(id: usize, log_n: usize, terminate_round: usize) -> AvssNode {
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
        // for i in 0..folding_parameter.len() {
        //     println!("{}: {:?}", i, folding_parameter[i]);
        // }

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

        let dealer = Dealer::new(
            log_d - terminate_round,
            &polynomial,
            &interpolate_cosets,
            &oracle,
            &folding_parameter,
        );

        AvssNode {
            id: id,
            log_n: log_n,
            terminate_round: terminate_round,
            polynomial: polynomial,
            dealer: dealer,
            parties: parties,
        }

    }
    
    pub fn send_and_verify(&mut self, msg_type: MessageType) -> Option<Message> {
        self.dealer.send_evaluations(&mut self.parties);
        self.dealer.commit_functions(&self.parties);
        self.dealer.prove();
        self.dealer.commit_foldings(&self.parties);
        let (folding, function) = self.dealer.query();
        let mut folding0 = vec![];
        let mut function0 = vec![];

        let log_d = (self.log_n - 2) * 2;
        let terminate_round = self.terminate_round;

        for i in 0..(log_d - terminate_round) {
            if i < log_d - terminate_round - 1 {
                folding0.push(folding[i][0].clone());
            }
            function0.push(function[i][0].clone());
        }
        
        assert!(self.parties[0].verify(&folding0, &function0));

        Message::send_message2all(self.id, msg_type, vec![])
    }

    pub fn shares(&self) -> Vec<Vec<Mersenne61Ext>> {
        let mut shares = vec![];
        let n = 1 << self.log_n;
        for i in 0..n {
            shares.push(self.parties[i*n].interpolate_share());
            // println!("shares {}: {:?}", i, shares[i])
        }
        shares
    }

    pub fn sum_and_rec(&self, dealers: Vec<usize>) -> Mersenne61Ext{
        let mut sum = Mersenne61Ext::from_int(0);
        for i in 0..dealers.len() {
            sum += self.parties[dealers[i]].share();
        }
        sum
    } 

    pub fn reconstruct(&self) -> Mersenne61Ext{
        // let n = 1 << self.log_n;

        // println!("{}", self.parties.len());
        // for i in 0..self.parties.len() {
        //     println!("reconstruct {}: {:?}", i, self.parties[i].all_share().evaluate_as_polynomial(Mersenne61Ext::from_int(0)));
        // }
        // println!(" ==================================== ");
        
        // for i in 0..n {
        //     println!("reconstruct {}: {:?}", i, self.parties[i*n].all_share().evaluate_as_polynomial(Mersenne61Ext::from_int(0)));
        // }

        if self.parties[0].has_share() {
            self.parties[0].all_share().evaluate_as_polynomial(Mersenne61Ext::from_int(0))
        }else {
            Mersenne61Ext::from_int(0)
        }
    }

    pub fn get_poly(&self) -> MultilinearPolynomial<Mersenne61Ext> {
        self.polynomial.clone()
    }
    
}


#[cfg(test)]
mod tests {
    use super::AvssNode;
    use crate::msg::message::MessageType;

    #[test]
    fn avss_log_print() {
        let mut s = AvssNode::new(0, 3, 1);
        let m = s.send_and_verify(MessageType::AdkgAvssFin);
        println!("{}", m.unwrap());

        let shares = s.shares();
        println!("{:?}", shares);
        println!("{}", shares.len());
        let res = s.reconstruct();
        println!("{}", res);

    }
}

