use std::collections::HashMap;
use super::utils::{is_invector, is_subset};

#[derive(Clone)]
#[derive(Debug)]
pub struct Message {
    pub sender_id: usize,
    pub msg_type: usize,
    data: Vec<usize>,
    additional_data: String,
}

#[derive(Debug)]
pub struct Node {
    pub id: usize,
    pub msg: String,
    state: usize,                   // 0: adversary, 1: honest
    set_r: Vec<(usize, String)>,
    set_s: Vec<usize>,
    set_t: Vec<usize>,
    set_u: Vec<usize>,
    n: usize,
    f: usize,
    others_s_set: HashMap<usize, Vec<usize>>,
    pub msg_queue: Vec<Message>,
}

impl Node {
    pub fn new(id: usize, n: usize, f: usize, state: usize) -> Node {
        Node {
            id,
            msg: format!("Node id: {}", id),
            state,
            set_r: Vec::new(),
            set_s: Vec::new(),
            set_t: Vec::new(),
            set_u: Vec::new(),
            n,
            f,
            others_s_set: HashMap::new(),
            msg_queue: Vec::new(),
        }
    }

    pub fn send_1_message(&self) -> Option<Message>{
        if self.state == 0 {
            return None
        }

        println!("Node {} send type 1 message: {:?}", self.id, self.msg);
        Some(Message {
            sender_id: self.id,
            msg_type: 1,
            data: Vec::new(),
            additional_data: self.msg.clone(),
        }) 
    }

    /// 处理消息队列中的消息
    pub fn handle_msg_queue(&mut self, get_message: Option<Message>) -> Option<Message>{
        // println!("node condition: {:?}", self);
        match get_message {
            Some(message) => self.msg_queue.push(message),
            None => (),
        }

        while !self.msg_queue.is_empty() {
            let message = self.msg_queue.pop().unwrap();
            let new_message = self.handle_message(message);
            match new_message {
                Some(m) => {
                    if m.msg_type == 4 { 
                        println!("@@@@ Node {} get type 4 message: {:?}", self.id, m); 
                        return None
                    }
                    else {
                        return Some(m)
                    }
                }
                None => return None,
            }
                
        }
        None
    }
    

    /// 处理单个消息
    pub fn handle_message(&mut self, message: Message) -> Option<Message>{
        if self.state == 0 {
            return None
        }

        match message.msg_type {
            1 => self.handle_type_1_message(message),
            2 => self.handle_type_2_message(message),
            3 => self.handle_type_3_message(message),
            4 => Some(message),
            _ => {println!("Error: message type error!"); None},
        }
    }

    /// 消息1中data字段舍弃，将 sendder_id 和 xj(additional_data) 存入 Ri 中，将 sender_id 存入 Si 中
    // fn handle_type_1_message(&mut self, message: Message, nodes: &mut HashMap<usize, Node>) {
    fn handle_type_1_message(&mut self, message: Message) -> Option<Message>{

        if !is_invector(message.sender_id, &self.set_s) {
            self.set_r.push((message.sender_id, message.additional_data));
            self.set_s.push(message.sender_id);
        }else {
            return None
        }

        let n_f = self.n - self.f;
        if self.set_s.len() >= n_f {
            Some(Message {
                sender_id: self.id,
                msg_type: 2,
                data: self.set_s.clone(),
                additional_data: String::new(),
            })
        }else {
            None
        }
    }

    fn handle_type_2_message(&mut self, message: Message) -> Option<Message>{
        self.others_s_set.insert(message.sender_id, message.data.clone());

        if is_subset(&message.data, &self.set_s) {
            self.set_t.push(message.sender_id);
        }else {
            return None;
        }

        let n_f = self.n - self.f;
        if self.set_t.len() >= n_f {
            Some(Message {
                sender_id: self.id,
                msg_type: 3,
                data: self.set_t.clone(),
                additional_data: String::new(),
            })
        }else {
            None
        }
    }

    fn handle_type_3_message(&mut self, message: Message) -> Option<Message>{
        if !is_invector(self.id, &self.set_u) {
            self.set_u.push(self.id);
        }

        if is_subset(&message.data, &self.set_t) {
            for user in message.data {
                let sk = self.others_s_set.get(&user).unwrap();
                for &id in sk{
                    if !is_invector(id, &self.set_u) {
                        self.set_u.push(id);
                    }
                }
            }
        }

        let n_f = self.n - self.f;
        println!("Node {} set_u: {:?}", self.id, self.set_u);
        if self.set_u.len() >= n_f {
            Some(Message {
                sender_id: self.id,
                msg_type: 4,
                data: self.set_s.clone(),
                additional_data: String::new(),
            })
        }else {
            None
        }
    }

}


