use rand;
use std::collections::HashMap;

use util::vec_check::{is_invector, is_subset};
use super::gather::GatherNode;
use crate::server::message::Message;
use crate::server::message::{VABA_ATTACH, VABA_SIG, VABA_INDICE, VABA_EVAL, VABA_FIN};


#[derive(Debug)]
pub struct VabaNode {
    id: usize,
    state: usize,
    f: usize,
    secret: Vec<usize>,
    set_dealer: Vec<usize>,
    set_attached: Vec<usize>,
    set_sig: Vec<(usize, Message)>,
    set_indice: Vec<usize>,
    set_fin: HashMap<usize, usize>,
    pub gather: GatherNode,
    pub fin: (usize, usize),
}

impl VabaNode {
    /// new 新建一个 VABA 节点，其中包含所需的 gather 节点
    pub fn new(id: usize, state: usize, n: usize, f: usize) -> VabaNode {
        VabaNode {
            id,
            state,
            f,
            secret: (0..n).map(|_| rand::random::<usize>()).collect(),
            set_dealer: Vec::new(),
            set_attached: Vec::new(),
            set_sig: Vec::new(),
            set_indice: Vec::new(),
            set_fin: HashMap::new(),
            gather: GatherNode::new(id, n-f, state),
            fin: (0, 0),
        }
    }

    pub fn send_message(&self, recv: Vec<usize>, msg_type: usize, msg_content: Vec<usize>) -> Option<Message>{
        if self.state == 0 {
            return None
        }
        Some(Message { 
            sender_id: self.id,
            receiver_id: recv,
            msg_type: msg_type,
            msg_content: msg_content.clone(),
            additional: String::new(),
        })
    }
    
    /// 作为 Dealer 进行秘密分享，当完成其他参与者的 Share 过程时，将其添加到 set_dealer 中
    /// 当 set_dealer 中的参与者数量达到 f+1 时，将 set_dealer 赋值给 set_attached，并发送消息 <VABA_ATTACH>
    /// 这里进行模拟，表示参数 id 的参与者已经完成了 Share 过程，将其添加到 set_dealer 中
    pub fn handle_start(&mut self, id: usize) -> Option<Message> {
        if is_invector(id, &self.set_dealer) {
            return None
        }
        self.set_dealer.push(id);

        if self.set_dealer.len() == self.f + 1 {
            self.set_attached = self.set_dealer.clone();
            return self.send_message(vec![], VABA_ATTACH, self.set_attached.clone());
        }
        None
    }

    /// 收到消息 <VABA_ATTACH> 后，判断与 set_dealer 的子集关系，如果是，则为其签名，并向其发送消息 <VABA_SIG>
    pub fn handle_attach(&mut self, msg: Message) -> Option<Message> {
        // msg.msg_content 是自己的 set_dealer 集合的子集
        if !is_subset(&msg.msg_content, &self.set_dealer) {
            return None
        }

        let mut message = self.send_message(vec![msg.sender_id], VABA_SIG, msg.msg_content.clone()).unwrap();
        message.additional = format!("The set of dealer is {:?}, which is SIGNATURED by {}", msg.msg_content, self.id);
        Some(message)
    }

    /// 收到他人的签名信息 <VABA_SIG> 后，将其添加到 set_sig 中
    /// 如果 set_sig 中的签名数量达到 f+1，则调用 GatherStart 进行求交
    pub fn handle_sig(&mut self, msg: Message) -> Option<Message> {
        if msg.msg_content.len() == 0 || self.set_attached.len() == 0 {
            return None
        }
        let number =  msg.additional.trim_end_matches(|c: char| !c.is_digit(10));

        if let Ok(number) = number.parse::<usize>() {
            if number == self.id {
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
            return self.gather.start();
        }
        None
    }

    /// 收到 Gather 的消息 <GATHER_FIN> 后，发回消息，并将 Gather 的结果赋值给 set_indice
    /// 发送消息 <VABA_INDICE>，并将 set_indice 作为消息内容，待其他人对其进行验证
    pub fn handle_gather_fin(&mut self, msg: Message) -> Option<Message> {
        self.set_indice = msg.msg_content.clone();
        self.send_message(vec![], VABA_INDICE, self.set_indice.clone())
    }

    /// 收到其他人的验证消息 <VABA_INDICE> 后，调用 GatherVerify 进行验证
    /// 如果自己的 id 在 Gather 输出中，则调用 BingoReconstructSum 并输出结果，并通过消息 <VABA_EVAL> 发送
    /// 这里进行模拟，随机产生BingoReconstructSum 结果
    pub fn handle_indice(&mut self, msg: Message) -> Option<Message> {
        // 调用 GatherVerify 进行验证
        if self.gather.verify(msg.msg_content.clone()) {
            if msg.msg_content.contains(&self.id) {
                // 调用 BingoReconstructSum 并输出结果
                // 计算 self.secret 的和
                let mut sum = 0;
                for i in 0..self.secret.len() {
                    sum += self.secret[i];
                }
                return self.send_message(vec![], VABA_EVAL, vec![sum])
            }
        }
        None
    }

    /// 如果收到消息 <VABA_EVAL>，则将其添加到 set_fin 中
    /// 如果 set_indice 中的参与者都已经重构出秘密，则将 set_fin 中的最大值作为结果，通过消息 <VABA_FIN> 发送
    pub fn handle_eval(&mut self, msg: Message) -> Option<Message> {
        if self.set_indice.contains(&msg.sender_id){
            if !self.set_fin.contains_key(&msg.sender_id){
                let s = msg.msg_content[0];
                if s > self.fin.1 {
                    self.fin = (msg.sender_id, s);
                }
            }
        }
        if self.set_fin.len() == self.set_indice.len() {
            return self.send_message(vec![], VABA_FIN, vec![self.fin.0, self.fin.1])
        }
        None
    }

    
}