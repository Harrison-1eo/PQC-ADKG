use std::collections::HashMap;
use lazy_static::lazy_static;
use super::utils::{is_invector, is_subset};

#[derive(Clone)]
struct Message {
    sender_id: usize,
    msg_type: usize,
    data: Vec<usize>,
    additional_data: String,
}

pub struct Node {
    id: usize,
    msg: String,
    status: usize,                      // 0: bad, 1: good
    set_r: Vec<(usize, String)>,
    set_s: Vec<usize>,
    set_t: Vec<usize>,
    set_u: Vec<usize>,
    n: usize,
    f: usize,
    nodes: &'static HashMap<usize, Node>
}

lazy_static! {
    static ref NODES: HashMap<usize, Node> = {
        let mut map = HashMap::new();
        map.insert(1, Node::new(1, 1));
        map
    };
}

impl Node {
    pub fn new(id: usize, status: usize) -> Node {
        Node {
            id,
            msg: format!("Node id: {}", id),
            status,
            set_r: Vec::new(),
            set_s: Vec::new(),
            set_t: Vec::new(),
            set_u: Vec::new(),
            n: 10,
            f: 3,
            nodes: { &NODES },
        }
    }

    pub fn send_1_message(&self) {
        let msg = Message {
            sender_id: self.id,
            msg_type: 1,
            data: Vec::new(),
            additional_data: self.msg.clone(),
        };
        self.broadcast_message(msg);
    }

    // fn handle_message(&mut self, message: Message, nodes: &mut HashMap<usize, Node>) {
    fn handle_message(&mut self, message: Message) {
        match message.msg_type {
            1 => self.handle_type_1_message(message),
            2 => self.handle_type_2_message(message),
            3 => self.handle_type_3_message(message),
            _ => println!("Invalid message type"),
        }
    }

    /// 消息1中data字段舍弃，将 sendder_id 和 xj(additional_data) 存入 Ri 中，将 sender_id 存入 Si 中
    // fn handle_type_1_message(&mut self, message: Message, nodes: &mut HashMap<usize, Node>) {
    fn handle_type_1_message(&mut self, message: Message) {
        if is_invector(&message.sender_id, &self.set_s) {
            return;
        }

        self.set_r.push((message.sender_id, message.additional_data));

        let n_f = self.n - self.f;
        if self.set_r.len() == n_f {
            let msg = Message {
                sender_id: self.id,
                msg_type: 2,
                data: self.set_s.clone(),
                additional_data: String::new(),
            };
            self.broadcast_message(msg);
        }
    }

    fn handle_type_2_message(&mut self, message: Message) {
        if is_invector(&message.sender_id, &self.set_s) {
            return;
        }

        if is_subset(&message.data, &self.set_s) {
            self.set_t.push(message.sender_id);
        }

        let n_f = self.n - self.f;
        if self.set_t.len() == n_f {
            let msg = Message {
                sender_id: self.id,
                msg_type: 3,
                data: self.set_t.clone(),
                additional_data: String::new(),
            };
            self.broadcast_message(msg);
        }
    }

    fn handle_type_3_message(&mut self, message: Message) {
        if !is_invector(&self.id, &self.set_u) {
            self.set_u.push(self.id);
        }

        if is_subset(&message.data, &self.set_t) {
            for user in message.data {
                let sk = self.nodes.get(&user).unwrap().set_s.clone();
                for id in sk{
                    if !is_invector(&id, &self.set_u) {
                        self.set_u.push(id);
                    }
                }
            }
        }
        
    }

    // fn broadcast_message(&self, msg: Message, nodes: &mut HashMap<usize, Node>) {

        
    fn broadcast_message(&self, msg: Message) {
        for (id, node) in &mut *self.nodes {
            if id != &self.id {
                // let send_nodes = nodes.clone();
                node.handle_message(msg.clone());
            }
        }
    }

    // Implement similar functions for handling type 2 and type 3 messages
}


