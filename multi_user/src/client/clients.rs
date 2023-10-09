use super::vaba::VabaNode;
use crate::server::message::{Message, self};
use crate::server::message::{GATHER_1, GATHER_2, GATHER_3, GATHER_FIN};
use crate::server::message::{VABA_ATTACH, VABA_SIG, VABA_INDICE, VABA_EVAL, VABA_FIN};


pub struct Client {
    pub id: usize,
    pub state: usize,
    pub n: usize,
    pub f: usize,
    pub additional_data: String,
    vaba: VabaNode,
}

impl Client {
    pub fn new(id: usize, state: usize, n: usize, f: usize) -> Client {
        Client {
            id,
            state,
            n,
            f,
            additional_data: String::new(),
            vaba: VabaNode::new(id, state, n, f),
        }
    }

    pub fn start(&mut self) -> Option<Message> {
        self.gather_start()
    }

    pub fn end(&mut self){
        println!("Client {} end", self.id);
        println!("find {} at max {}", self.vaba.fin.0, self.vaba.fin.1);
    }

    pub fn handle_message(&mut self, msg: Message) -> Option<Message> {
        if self.state == 0 {
            return None
        }
        let mut message;

        if msg.msg_type == VABA_ATTACH {
            message = self.vaba.handle_attach(msg)
        }else if msg.msg_type == VABA_SIG {
            message = self.vaba.handle_sig(msg)
        }else if msg.msg_type == VABA_INDICE {
            message = self.vaba.handle_indice(msg)
        }else if msg.msg_type == VABA_EVAL {
            message = self.vaba.handle_eval(msg)
        }else if msg.msg_type == GATHER_1 {
            message = self.vaba.gather.handle_gather_1(msg)
        }else if msg.msg_type == GATHER_2 {
            message = self.vaba.gather.handle_gather_2(msg)
        }else if msg.msg_type == GATHER_3 {
            message = self.vaba.gather.handle_gather_3(msg)
        }else {
            return None;
        }

        match message {
            Some(m) => {
                if m.msg_type == VABA_FIN {
                    self.end();
                    None
                }else if m.msg_type == GATHER_FIN {
                    self.vaba.handle_gather_fin(m)
                }else {
                    Some(m)
                }
            },
            None => None,
        }
    }

    pub fn gather_start(&self) -> Option<Message> {
        if self.state == 0 {
            return None
        }
        self.vaba.gather.send_message(GATHER_1, vec![])
    }

    pub fn vaba_start(&mut self) -> Option<Message> {
        // 作为 dealer 分三次调用 BingoShare 分享n个秘密
        // 对于其他参与者 j 发来的三次 BingoShare 请求都进行参与

        // 如果完成了某个参与者三次的BingoShare 请求
        self.vaba.handle_start(1)
    }

}