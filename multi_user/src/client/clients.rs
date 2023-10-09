use super::gather::GatherNode;
use super::super::server::message::{Message};
use super::super::server::message::{GATHER_1, GATHER_2, GATHER_3, GATHER_FIN};
use super::super::server::message::{VABA_ATTACH, VABA_SIG, VABA_INDICE, VABA_FIN};


pub struct Client {
    pub id: usize,
    pub state: usize,
    pub additional_data: String,
    pub n: usize,
    pub f: usize,
    gather_node: GatherNode,

}

impl Client {
    pub fn new(id: usize, state: usize, n: usize, f: usize) -> Client {
        Client {
            id,
            state,
            additional_data: String::new(),
            n,
            f,
            gather_node: GatherNode::new(id, n-f, state),
        }
    }

    pub fn start(&mut self) -> Option<Message> {
        if self.state == 0 {
            return None
        }

        self.gather_node.send_message(GATHER_1, vec![])
    }

    pub fn handle_message(&mut self, msg: Message) -> Option<Message> {
        if self.state == 0 {
            return None
        }

        if msg.msg_type == GATHER_1 {
            self.gather_node.handle_gather_1(msg)
        }else if msg.msg_type == GATHER_2 {
            self.gather_node.handle_gather_2(msg)
        }else if msg.msg_type == GATHER_3 {
            self.gather_node.handle_gather_3(msg)
        }else if msg.msg_type == GATHER_FIN {
            // println!("Node {} receive gather fin message from {} \n{}", self.id, msg.sender_id, msg);
            None
        }else {
            None
        }

    }

}