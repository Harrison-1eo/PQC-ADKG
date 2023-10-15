use std::collections::HashMap;
use std::sync::mpsc;
use super::message::Message;
use crate::client::clients::Client;

pub struct BroadcastServer {
    pub n: usize,
    pub rx_from_threads: mpsc::Receiver<Message>,
    pub tx_to_threads: HashMap<usize, mpsc::Sender<Message>>,
}

pub struct UserThread {
    pub thread_id: usize,
    pub tx_to_server: mpsc::Sender<Message>,
    pub rx_from_server: mpsc::Receiver<Message>,
    pub client: Client,
}

impl BroadcastServer {
    fn broadcast2all(&self, msg: Message){
        for i in 0..self.n {
            if let Some(tx) = self.tx_to_threads.get(&i) {
                tx.send(msg.clone()).unwrap();
            }
        }
    }

    fn send2some(&self, msg: Message) {
        let mut recv = msg.receiver_id.clone();
        // recv排序后去除重复元素
        recv.sort();
        recv.dedup();
        // 如果存在大于n的接收者id，去除
        recv.retain(|&x| x < self.n);

        for i in recv {
            if let Some(tx) = self.tx_to_threads.get(&i) {
                tx.send(msg.clone()).unwrap();
            }
        }
    }

    pub fn send_msg(&self, msg: Message) {
        let len = msg.receiver_id.len();
        if len == 0 {
            self.broadcast2all(msg);
        } else {
            self.send2some(msg);
        }
    }
}