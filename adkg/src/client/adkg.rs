// use sha256::digest;
use std::collections::HashMap;
use util::vec_check::{is_invector, is_subset};
use super::avss::AvssNode;
use crate::msg::result::AdkgResult;
use crate::msg::message::Message;
use crate::msg::message::MessageType;


pub struct AdkgNode {
    id: usize,
    state: usize,
    n: usize,
    f: usize,
    set_dealer: Vec<usize>,
    set_prop: Vec<usize>,
    hash_prop: HashMap<usize, Vec<usize>>,
    set_sig: Vec<(usize, Message)>,
    fin: bool,
    set_fin: Vec<usize>,
    hash_fin: HashMap<usize, u64>,
    avss: AvssNode,
    pub res: Option<AdkgResult>,
    
}

impl AdkgNode {
    pub fn new(id: usize, state: usize, n: usize, f: usize) -> AdkgNode {
        AdkgNode {
            id,
            state,
            n,
            f,
            set_dealer: Vec::new(),
            set_prop: Vec::new(),
            hash_prop: HashMap::new(),
            set_sig: Vec::new(),
            fin: false,
            set_fin: Vec::new(),
            hash_fin: HashMap::new(),
            avss: AvssNode::new(id, log_2_n(n), 1),
            res: None,
        }
    }

    pub fn send_message(&self, recv: Vec<usize>, msg_type: MessageType, msg_content: Vec<usize>) -> Option<Message>{
        Message::send_message(self.id, recv, msg_type, msg_content)
    }

    pub fn start(&mut self) -> Option<Message>{
        if self.state == 0 {
            return None
        }
        println!("client_id:{} status:ADKG_SHARE_FIN", self.id);
        self.avss.send_and_verify(MessageType::AdkgAvssFin)
    }

    pub fn handle_share_fin(&mut self, id: usize) -> Option<Message> {
        if is_invector(id, &self.set_dealer) {
            return None
        }
        self.set_dealer.push(id);

        if self.set_dealer.len() == self.f + 1 {
            self.set_prop = self.set_dealer.clone();
            return self.send_message(vec![], MessageType::AdkgProp, self.set_prop.clone());
        }
        None
    }

    pub fn handle_prop(&mut self, msg: Message) -> Option<Message> {
        if !self.hash_prop.contains_key(&msg.sender_id) {
            self.hash_prop.insert(msg.sender_id, msg.msg_content.clone());
        }

        // msg.msg_content 是自己的 set_dealer 集合的子集
        if !is_subset(&msg.msg_content, &self.set_dealer) {
            return None
        }

        let mut message = self.send_message(vec![msg.sender_id], MessageType::AdkgSig, msg.msg_content.clone()).unwrap();
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

        if self.set_sig.len() == self.f + 1 {
            println!("client_id:{} status:ADKG_SIG_ENOUGH set:{:?}", self.id, self.set_prop);
            return self.send_message(vec![self.id], MessageType::VabaStart, self.set_prop.clone());
        }
        None
    }

    pub fn handle_vaba_fin(&mut self, msg: Message) -> Option<Message>{

        self.set_fin = self.hash_prop.get(&msg.msg_content[0]).unwrap().clone();
        self.fin = true;
        println!("client_id:{} status:ADKG_FIN set:{:?}", self.id, self.set_fin);
        return Some(Message::send_message_with_addi(
            self.id, 
            vec![], 
            MessageType::SumAndRec,
            vec![], 
            self.avss.sum_and_rec(self.set_fin.clone()).get_real().to_string().clone(),
        ));
    }

    pub fn sum_and_rec(&mut self, msg: Message) -> Option<AdkgResult> {

        self.hash_fin.insert(msg.sender_id, msg.additional.parse::<u64>().unwrap());
        // println!("sum_and_rec, {}, {:?}, {:?}", self.id, self.hash_fin.keys(),self.set_fin);
        let n_f: u64 = (self.n - self.f) as u64; 
        if self.hash_fin.len() == self.n - self.f {
            let mut sum: u64 = 0;
            for (_, &v) in self.hash_fin.iter() {
                sum += v / n_f;
            }
            return Some(AdkgResult {
                id: self.id,
                users: self.set_fin.clone(),
                sk: self.avss.reconstruct().get_real().to_string().clone(),
                pk: sum.to_string(),
            })
        }
        None
    }


}

/// 计算log2(n)
fn log_2_n (n: usize) -> usize {
    let mut i = 0;
    let mut tmp = n;
    while tmp > 1 {
        tmp = tmp >> 1;
        i += 1;
    }
    if 1<<i == n {
        i
    } else {
        i+1
    }
}

#[cfg(test)]
mod tests {
    fn log_2_n (n: usize) -> usize {
        let mut i = 0;
        let mut tmp = n;
        while tmp > 1 {
            tmp = tmp >> 1;
            i += 1;
        }
        if 1<<i == n {
            i
        } else {
            i+1
        }
    }


    #[test]
    fn t() {
        for i in 3..70 {
            println!("{}: {}", i, log_2_n(i));
        }
    }
}
