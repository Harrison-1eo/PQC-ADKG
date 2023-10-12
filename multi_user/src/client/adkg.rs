use rand::Rng;
use sha256::digest;

use util::vec_check::{is_invector, is_subset};
use super::vaba::VabaNode;
use super::res::Result;
use crate::server::message::Message;
use crate::server::message::{ADKG_PROP, ADKG_SIG};


#[derive(Debug)]
pub struct AdkgNode {
    id: usize,
    state: usize,
    f: usize,
    secret: Vec<usize>,
    set_dealer: Vec<usize>,
    set_prop: Vec<usize>,
    set_sig: Vec<(usize, Message)>,
    fin: bool,
    set_fin: Vec<usize>,
    pub vaba: VabaNode,
    pub res: Option<Result>,
    
}

impl AdkgNode {
    pub fn new(id: usize, state: usize, n: usize, f: usize) -> AdkgNode {
        AdkgNode {
            id,
            state,
            f,
            secret: (0..n).map(|_| rand::thread_rng().gen_range(1..usize::MAX/n)).collect(),
            set_dealer: Vec::new(),
            set_prop: Vec::new(),
            set_sig: Vec::new(),
            fin: false,
            set_fin: Vec::new(),
            vaba: VabaNode::new(id, state, n, f),
            res: None,
        }
    }

    pub fn send_message(&self, recv: Vec<usize>, msg_type: usize, msg_content: Vec<usize>) -> Option<Message>{
        self.vaba.send_message(recv, msg_type, msg_content)
    }

    pub fn handle_share_fin(&mut self, id: usize) -> Option<Message> {
        if is_invector(id, &self.set_dealer) {
            return None
        }
        self.set_dealer.push(id);

        if self.set_dealer.len() == self.f + 1 {
            self.set_prop = self.set_dealer.clone();
            return self.send_message(vec![], ADKG_PROP, self.set_prop.clone());
        }
        None
    }

    pub fn handle_prop(&mut self, msg: Message) -> Option<Message> {
        // msg.msg_content 是自己的 set_dealer 集合的子集
        if !is_subset(&msg.msg_content, &self.set_dealer) {
            return None
        }

        let mut message = self.send_message(vec![msg.sender_id], ADKG_SIG, msg.msg_content.clone()).unwrap();
        message.additional = format!("The set of dealer is {:?}, which is SIGNATURED by {}", msg.msg_content, self.id);
        Some(message)
    }
    

    pub fn handle_sig(&mut self, msg: Message) -> Option<Message> {
        if msg.msg_content.len() == 0 || self.set_prop.len() == 0 {
            return None
        }
        let number =  msg.additional.chars().rev().take_while(|c| c.is_digit(10))
                                                    .collect::<String>().chars().rev()
                                                    .collect::<String>().parse::<usize>();

        if let Ok(number) = number {
            if number == msg.sender_id {
                self.set_sig.push((msg.sender_id, msg));
            }
            else {
                println!("The signature is not valid!");
                return None
            }
        } else {
            println!("No valid signed number found!");
            return None
        }

        // if self.set_sig.len() == self.f + 1 {
        //     return self.vaba.start();
        // }
        None
    }

    pub fn handle_vaba_fin(&mut self, msg: Message) -> Option<Result>{
        self.set_fin = msg.msg_content.clone();
        self.fin = true;

        if is_invector(self.id, &self.set_fin) {
            self.res = self.sum_and_rec(msg.msg_content.clone());
        }

        self.res.clone()
    }

    fn sum_and_rec(&self, users:Vec<usize>) -> Option<Result> {
        // 对 self.id 做SHA256运算
        let sk = digest(self.id.to_string());
        // 生成一个随机字符串
        let ram = rand::thread_rng().gen_range(1..usize::MAX/1000).to_string();
        let mut pk = digest(ram);
        for &user in users.iter() {
            let tmp = digest(user.to_string());
            // pk = pk ^ tmp, pk is String type
            pk = pk.chars().zip(tmp.chars()).map(|(a, b)| (a as u8 ^ b as u8) as char).collect::<String>();
        }

        Some(Result { 
            id: self.id, 
            users: users.clone(),
            sk: sk.clone(),
            pk: pk.clone(),
        })

    }
}


#[cfg(test)]
mod tests {
    use rand::Rng;
    use sha256::digest;
    use super::super::res::Result;
    
    fn sum_and_rec(id:usize, users:Vec<usize>) -> Option<Result> {
        // 对 self.id 做SHA256运算
        let sk = digest(id.to_string());
        let mut pk = digest(0.to_string());
        for &user in users.iter() {
            let tmp = digest(user.to_string());
            // pk = pk ^ tmp, pk is String type
            pk = pk.chars().zip(tmp.chars()).map(|(a, b)| (a as u8 ^ b as u8) as char).collect::<String>();
        }

        Some(Result { 
            id: id, 
            users: users.clone(),
            sk: sk.clone(),
            pk: pk.clone(),
        })

    }

    #[test]
    fn t() {
        let mut users: Vec<usize> = Vec::new();
        for i in 0..20 {
            // 在0和1之间随机生成一个数
            if rand::thread_rng().gen_range(0..2)==1 {
                users.push(i);
            }
        }
        for i in 0..20 {
            let res = sum_and_rec(i, users.clone());
            println!("{}", res.unwrap());
        }
    }
}
