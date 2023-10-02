use super::users::{Message, Node};
use std::collections::HashMap;

pub struct BroadcastServer {
    nodes: HashMap<usize, Node>,
    n: usize,
    // f: usize,
    msg_queue: Vec<Message>,
}

impl BroadcastServer {
    pub fn new(n: usize, f: usize) -> BroadcastServer {
        let mut nodes = HashMap::new();
        for i in 0..(n-f) {
            nodes.insert(i, Node::new(i, n, f, 1));
        }
        for i in (n-f)..n {
            nodes.insert(i, Node::new(i, n, f, 0));
        }

        BroadcastServer {
            nodes,
            n,
            // f,
            msg_queue: Vec::new(),
        }
    }

    pub fn start(&mut self) {
        for i in 0..self.n {
            let message: Option<Message> = self.nodes.get(&i).unwrap().send_1_message();
            if message.is_some() {
                self.msg_queue.push(message.unwrap());
            }
        }
    }

    pub fn broadcast2all(&mut self) {
        let mut flag = true;
        while flag {
        
            // 如果server的消息队列不为空，则处理server消息队列中的消息
            if !self.msg_queue.is_empty() {
                let message = self.msg_queue.pop().unwrap();
            
                for i in 0..self.n {
                    // if i != message.sender_id {
                        let new_message = self.nodes.get_mut(&i).unwrap().
                                                                handle_msg_queue(Some(message.clone()));
                        println!("node {} get message: {:?}", i, message);
                        
                        match new_message {
                            Some(msg) => {
                                println!("server get new message: {:?}", msg);
                                self.msg_queue.push(msg)
                            },
                            None => (),
                        }
                    // }
                }
            }

            // 如果server的消息队列为空，则处理node消息队列中的消息
            for i in 0..self.n {
                let node = self.nodes.get_mut(&i).unwrap();
                if !node.msg_queue.is_empty() {
                    let new_message = node.handle_msg_queue(None);
                    match new_message {
                        Some(msg) => {
                            println!("server get new message: {:?}", msg);
                            self.msg_queue.push(msg)
                        },
                        None => (),
                    }
                }
            }

            // 如果server的消息队列为空，且每个node的消息队列也为空，则结束循环
            if self.msg_queue.is_empty() {
                flag = false;
                for i in 0..self.n {
                    if !self.nodes.get(&i).unwrap().msg_queue.is_empty() {
                        flag = true;
                    }
                }
            }
        }
    }
}
