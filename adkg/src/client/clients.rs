use super::vaba::VabaNode;
use super::gather::GatherNode;
use super::adkg::AdkgNode;
use crate::msg::message::{Message, MessageType};
use crate::msg::result::AdkgResult;
use std::time::Instant;

pub struct Client{
    pub id: usize,
    pub state: usize,
    pub n: usize,
    pub f: usize,
    pub additional_data: String,
    gather: GatherNode,
    vaba: VabaNode,
    adkg: AdkgNode,
    start_time: Instant,
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
            start_time: std::time::Instant::now(),
        }
    }

    pub fn start(&mut self) -> Option<Message> {
        if self.state == 0 {
            return None
        }

        self.adkg.start()
    }

    pub fn handle_message(&mut self, msg: Message) -> Option<Message> {
        if self.state == 0 {
            return None
        }

        let message = match msg.msg_type {
            MessageType::AdkgAvssFin   => self.adkg.handle_share_fin(msg.sender_id),
            MessageType::AdkgProp      => self.adkg.handle_prop(msg),
            MessageType::AdkgSig       => self.adkg.handle_sig(msg),
            MessageType::VabaAvssFin   => self.vaba.handle_share_fin(msg.sender_id),
            MessageType::VabaAttach    => self.vaba.handle_attach(msg),
            MessageType::VabaSig       => self.vaba.handle_sig(msg),
            MessageType::VabaIndice    => self.vaba.handle_indice(msg),
            MessageType::VabaEval      => self.vaba.handle_eval(msg),
            MessageType::Gather1       => self.gather.handle_gather_1(msg),
            MessageType::Gather2       => self.gather.handle_gather_2(msg),
            MessageType::Gather3       => self.gather.handle_gather_3(msg),
            MessageType::SumAndRec     => {
                let res = self.adkg.sum_and_rec(msg);
                match res {
                    Some(r) => self.end(r),
                    None => (),
                }
                None
            }
            
            _ => None,
        };

        match message {
            Some(m) => {
                // println!("{}", m);
                match m.msg_type {
                    MessageType::VabaStart      => self.vaba.start(),
                    MessageType::GatherStart    => self.gather.start(),
                    MessageType::GatherFin      => self.vaba.handle_gather_fin(m),
                    MessageType::VabaFin        => self.adkg.handle_vaba_fin(m),
                    _ => Some(m),
                }
            },
            None => None,
        }
    }

    pub fn end(&mut self, res: AdkgResult){
        // println!("Client {} end", self.id);
        // println!("{}", res);
        println!("client_id:{} status:GET_SK_PK sk:{} pk:{}", self.id, res.sk, res.pk);
        //结束进程
        println!("{}", self.start_time.elapsed().as_millis());
    }



}

