use super::vaba::VabaNode;
use super::gather::GatherNode;
use super::adkg::AdkgNode;
use super::avss::AvssNode;
use crate::msg::message::{Message, MessageType};


pub struct Client{
    pub id: usize,
    pub state: usize,
    pub n: usize,
    pub f: usize,
    pub additional_data: String,
    gather: GatherNode,
    vaba: VabaNode,
    adkg: AdkgNode,
    avss_adkg: AvssNode,
    avss_vaba: AvssNode,
}

impl Client {
    pub fn new(id: usize, state: usize, n: usize, f: usize) -> Client {
        Client {
            id,
            state,
            n,
            f,
            additional_data: String::new(),
            gather: GatherNode::new(id, state, n, f),
            vaba: VabaNode::new(id, state, n, f),
            adkg: AdkgNode::new(id, state, n, f),
            avss_adkg: AvssNode::new(id, 3, 1),
            avss_vaba: AvssNode::new(id, 3, 1),
        }
    }

    pub fn start(&mut self) -> Option<Message> {
        self.avss_adkg.send_and_verify(MessageType::AvssSendFin)
    }

    pub fn handle_message(&mut self, msg: Message) -> Option<Message> {
        if self.state == 0 {
            return None
        }

        let message = match msg.msg_type {
            MessageType::AvssSendFin   => self.adkg.handle_share_fin(msg.sender_id),
            MessageType::AdkgProp      => self.adkg.handle_prop(msg),
            MessageType::AdkgSig       => self.adkg.handle_sig(msg),
            MessageType::VabaSendFin   => self.vaba.handle_share_fin(msg.sender_id),
            MessageType::VabaAttach    => self.vaba.handle_attach(msg),
            MessageType::VabaSig       => self.vaba.handle_sig(msg),
            MessageType::VabaIndice    => self.vaba.handle_indice(msg),
            MessageType::VabaEval      => self.vaba.handle_eval(msg),
            MessageType::Gather1       => self.gather.handle_gather_1(msg),
            MessageType::Gather2       => self.gather.handle_gather_2(msg),
            MessageType::Gather3       => self.gather.handle_gather_3(msg),
            _ => None,
        };

        match message {
            Some(m) => {
                match m.msg_type {
                    MessageType::VabaStart => {
                        self.avss_vaba.send_and_verify(MessageType::VabaSendFin)
                    },
                    MessageType::GatherFin => {
                        self.vaba.handle_gather_fin(m)
                    }
                    MessageType::VabaFin => {
                        self.adkg.handle_vaba_fin(m);
                        None
                    },
                    _ => Some(m),
                }
            },
            None => None,
        }
    }

    pub fn end(&mut self){
        println!("Client {} end", self.id);
        println!("find {} at max {}", self.vaba.res.0, self.vaba.res.1);
    }

    // pub fn gather_start(&self) -> Option<Message> {
    //     if self.state == 0 {
    //         return None
    //     }
    //     self.vaba.gather.send_message(MessageType::Gather1, vec![])
    // }

    // pub fn vaba_start(&mut self) -> Vec<Option<Message>> {
    //     // 作为 dealer 分三次调用 BingoShare 分享n个秘密
    //     // 对于其他参与者 j 发来的三次 BingoShare 请求都进行参与

    //     // 如果完成了某个参与者三次的BingoShare 请求
    //     let mut m: Vec<Option<Message>> = vec![];
    //     for i in 0..self.n {
    //         m.push(self.vaba.handle_share_fin(i).clone())
    //     }
    //     m
    // }

}

