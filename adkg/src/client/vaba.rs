// use rand::Rng;
use std::collections::HashMap;

use util::vec_check::{is_invector, is_subset, is_equal};
// use util::algebra::field::mersenne61_ext::Mersenne61Ext;
use super::avss::AvssNode;
use crate::msg::message::Message;
use crate::msg::message::MessageType;

pub struct VabaNode {
    id: usize,
    state: usize,
    f: usize,
    // secret: Vec<usize>,
    set_dealer: Vec<usize>,
    set_attached: Vec<usize>,
    set_sig: Vec<(usize, Message)>,
    set_indice: Vec<usize>,
    set_fin: HashMap<usize, u64>,
    avss: AvssNode,
    pub res: (usize, u64),
    fin: bool,
}

impl VabaNode {
    /// new 新建一个 VABA 节点，其中包含所需的 gather 节点
    pub fn new(id: usize, state: usize, n: usize, f: usize) -> VabaNode {
        VabaNode {
            id,
            state,
            f,
            // secret: (0..n).map(|_| rand::thread_rng().gen_range(1..usize::MAX/n)).collect(),
            set_dealer: Vec::new(),
            set_attached: Vec::new(),
            set_sig: Vec::new(),
            set_indice: Vec::new(),
            set_fin: HashMap::new(),
            avss: AvssNode::new(id, log_2_n(n), 1),
            res: (0, 0),
            fin: false,
        }
    }

    pub fn send_message(&self, recv: Vec<usize>, msg_type:MessageType , msg_content: Vec<usize>) -> Option<Message>{
        if self.state == 0 {
            return None
        }

        Message::send_message(self.id, recv, msg_type, msg_content)
    }

    pub fn start(&mut self) -> Option<Message>{
        if self.state == 0 {
            return None
        }
        self.avss.send_and_verify(MessageType::VabaAvssFin)
    }
    
    /// 作为 Dealer 进行秘密分享，当完成其他参与者的 Share 过程时，将其添加到 set_dealer 中
    /// 当 set_dealer 中的参与者数量达到 f+1 时，将 set_dealer 赋值给 set_attached，并发送消息 <VABA_ATTACH>
    /// 这里进行模拟，表示参数 id 的参与者已经完成了 Share 过程，将其添加到 set_dealer 中
    pub fn handle_share_fin(&mut self, id: usize) -> Option<Message> {
        if is_invector(id, &self.set_dealer) {
            return None
        }
        self.set_dealer.push(id);

        if self.set_dealer.len() == self.f + 1 {
            self.set_attached = self.set_dealer.clone();
            return self.send_message(vec![], MessageType::VabaAttach, self.set_attached.clone());
        }
        None
    }

    /// 收到消息 <VABA_ATTACH> 后，判断与 set_dealer 的子集关系，如果是，则为其签名，并向其发送消息 <VABA_SIG>
    pub fn handle_attach(&mut self, msg: Message) -> Option<Message> {
        // msg.msg_content 是自己的 set_dealer 集合的子集
        if !is_subset(&msg.msg_content, &self.set_dealer) {
            return None
        }

        let mut message = self.send_message(vec![msg.sender_id], MessageType::VabaSig, msg.msg_content.clone()).unwrap();
        message.additional = format!("The set of dealer is {:?}, which is SIGNATURED by {}", msg.msg_content, self.id);
        Some(message)
    }

    /// 收到他人的签名信息 <VABA_SIG> 后，将其添加到 set_sig 中
    /// 如果 set_sig 中的签名数量达到 f+1，则调用 GatherStart 进行求交
    pub fn handle_sig(&mut self, msg: Message) -> Option<Message> {
        if msg.msg_content.len() == 0 || self.set_attached.len() == 0 {
            return None
        }
        let msg_content = msg.msg_content.clone();
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
            // return self.gather.start();
            return self.send_message(vec![self.id], MessageType::GatherStart, msg_content);
        }
        None
    }

    /// 收到 Gather 的消息 <GATHER_FIN> 后，发回消息，并将 Gather 的结果赋值给 set_indice
    /// 发送消息 <VABA_INDICE>，并将 set_indice 作为消息内容，待其他人对其进行验证
    pub fn handle_gather_fin(&mut self, msg: Message) -> Option<Message> {
        self.set_indice = msg.msg_content.clone();
        self.send_message(vec![], MessageType::VabaIndice, self.set_indice.clone())
    }

    /// 收到其他人的验证消息 <VABA_INDICE> 后，调用 GatherVerify 进行验证
    /// 如果自己的 id 在 Gather 输出中，则调用 BingoReconstructSum 并输出结果，并通过消息 <VABA_EVAL> 发送
    /// 这里进行模拟，随机产生BingoReconstructSum 结果
    pub fn handle_indice(&mut self, msg: Message) -> Option<Message> {
        // 调用 GatherVerify 进行验证
        if self.verify_indice(msg.msg_content.clone()) {
            if msg.msg_content.contains(&self.id) {
                // 调用 BingoReconstructSum 并输出结果
                // 计算 self.secret 的和
                // let mut sum: usize = 0;
                // for i in 0..self.secret.len() {
                //     sum = sum.wrapping_add(self.secret[i].into());
                // }

                let secret = self.avss.reconstruct().get_real().to_string();
                
                // return self.send_message(vec![], MessageType::VabaEval, vec![sum])
                return Some(Message::send_message_with_addi(
                    self.id, 
                    vec![], 
                    MessageType::VabaEval, 
                    vec![], 
                    secret.clone()
                ))
            }
        }
        None
    }

    /// 如果收到消息 <VABA_EVAL>，则将其添加到 set_fin 中
    /// 如果 set_indice 中的参与者都已经重构出秘密，则将 set_fin 中的最大值作为结果，通过消息 <VABA_FIN> 发送
    /// 消息中包含最大值的参与者 id 和最大值
    pub fn handle_eval(&mut self, msg: Message) -> Option<Message> {
        if self.fin {
            return None
        }
        if self.set_indice.contains(&msg.sender_id){
            let s = msg.additional.parse::<u64>().unwrap();
            self.set_fin.insert(msg.sender_id, s);
            if s > self.res.1 {
                self.res = (msg.sender_id, s);
            }
        }
        if self.set_fin.len() == self.set_indice.len() {
            self.fin = true;
            // return self.send_message(vec![], MessageType::VabaFin, vec![self.res.0]);
            return Some(Message::send_message_with_addi(
                self.id, 
                vec![], 
                MessageType::VabaFin, 
                vec![self.res.0], 
                msg.additional.clone()
            ))
        }
        else {
            print!("vaba.handle_eval: id: {}, self.set_fin: {:?}, self.set_indice: {:?}\n", self.id, self.set_fin.keys(), self.set_indice);
        }
        None
    }

    pub fn verify_indice(&self, indice: Vec<usize>) -> bool {
        is_equal(&indice, &self.set_indice)
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
    i+1
}

#[cfg(test)]
mod tests {

    #[test]
    fn str_cat() {
        let s = String::from("The set of dealer is [1412430, 2311], which is SIGNATURED by 07888");
        // 提取出s字符串最后的数字
        let last_num = s.chars().rev().take_while(|c| c.is_digit(10))
                                .collect::<String>().chars().rev()
                                .collect::<String>().parse::<i32>().unwrap();        
        println!("{}", last_num);
    }
}
