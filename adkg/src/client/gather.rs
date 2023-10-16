use util::vec_check::{is_invector, is_subset, is_equal};
use crate::msg::message::{Message, MessageType};


use std::collections::HashMap;


#[derive(Debug)]
pub struct GatherNode {
    id: usize,
    n_f: usize,
    state: usize,
    set_r: Vec<(usize, Vec<usize>)>,
    set_s: Vec<usize>,
    set_t: Vec<usize>,
    set_u: Vec<usize>,
    others_s_set: HashMap<usize, Vec<usize>>,
    fin: bool,
}

impl GatherNode {
    pub fn new(id: usize, state: usize, n: usize, f: usize) -> GatherNode {
        GatherNode {
            id,
            n_f: n-f,
            state,
            set_r: Vec::new(),
            set_s: Vec::new(),
            set_t: Vec::new(),
            set_u: Vec::new(),
            others_s_set: HashMap::new(),
            fin: false,
        }
    }

    pub fn start(&self) -> Option<Message> {
        if self.state == 0 {
            return None
        }
        self.send_message(MessageType::Gather1, vec![])
    }

    pub fn send_message(&self, msg_type: MessageType, msg_content: Vec<usize>) -> Option<Message>{
        if self.state == 0 {
            return None
        }

        Some(Message { 
            sender_id: self.id,
            receiver_id: vec![],
            msg_type: msg_type,
            msg_content: msg_content.clone(),
            additional: String::new(),
         })
    }

    pub fn handle_gather_1(&mut self, message: Message) -> Option<Message> {
        if !is_invector(message.sender_id, &self.set_s) {
            self.set_r.push((message.sender_id, message.msg_content.clone()));
            self.set_s.push(message.sender_id);
        }else {
            return None
        }

        if self.set_s.len() >= self.n_f {
            self.send_message(MessageType::Gather2, self.set_s.clone())
        }else {
            None
        }
    }

    pub fn handle_gather_2(&mut self, message: Message) -> Option<Message>{
        self.others_s_set.insert(message.sender_id, message.msg_content.clone());

        if is_subset(&message.msg_content, &self.set_s) &&
           !is_invector(message.sender_id, &self.set_t)  {
            self.set_t.push(message.sender_id);
        }else {
            return None;
        }

        if self.set_t.len() >= self.n_f {
            self.send_message(MessageType::Gather3, self.set_t.clone())
        }else {
            None
        }
    }

    pub fn handle_gather_3(&mut self, message: Message) -> Option<Message>{
        if !is_invector(self.id, &self.set_u) {
            self.set_u.push(self.id);
        }

        if is_subset(&message.msg_content, &self.set_t) {
            for user in message.msg_content {
                let sk = self.others_s_set.get(&user).unwrap();
                for &id in sk{
                    if !is_invector(id, &self.set_u) {
                        self.set_u.push(id);
                    }
                }
            }
        }

        // println!("Node {} set_u: {:?} ", self.id, self.set_u);
        if self.set_u.len() >= self.n_f {
            if self.fin {
                return None
            }
            self.fin = true;
            self.send_message(MessageType::GatherFin, self.set_s.clone())
        }else {
            None
        }
    }

    pub fn verify(&self, set_x: Vec<usize>) -> bool {
        is_equal(&set_x, &self.set_s)
    }

}