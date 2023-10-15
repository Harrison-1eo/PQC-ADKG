use std::thread;
use std::sync::mpsc;
use std::collections::HashMap;

use multi_user::server::servers::{BroadcastServer, UserThread};
use multi_user::client::clients::Client;

fn main() {
    let n: usize = 10;      // 假设有4个线程
    let f: usize = 3;        // 假设有1个拜占庭节点
    // 创建通道，用于线程向服务器发送消息
    let (tx_to_server, rx_to_server) = mpsc::channel();

    let mut threads = Vec::new();

    let mut server = BroadcastServer {
        n,
        rx_from_threads: rx_to_server,
        tx_to_threads: HashMap::new(),
    };
    print!("GOOD: ");
    for i in 0..(n-f) {
        print!(" {} ", i);
        let thread_id = i;

        let (tx_to_thread, rx_to_thread) = mpsc::channel();
        // thread_senders.insert(thread_id, tx_to_thread);
        server.tx_to_threads.insert(thread_id, tx_to_thread);

        threads.push(UserThread {
            thread_id,
            tx_to_server: tx_to_server.clone(),
            rx_from_server: rx_to_thread,
            client: Client::new(thread_id, 1, n, f),
        });
    }
    print!("BAD: ");
    for i in (n-f)..n {
        print!(" {} ", i);
        let thread_id = i;

        let (tx_to_thread, rx_to_thread) = mpsc::channel();
        // thread_senders.insert(thread_id, tx_to_thread);
        server.tx_to_threads.insert(thread_id, tx_to_thread);

        threads.push(UserThread {
            thread_id,
            tx_to_server: tx_to_server.clone(),
            rx_from_server: rx_to_thread,
            client: Client::new(thread_id, 0, n, f),
        });
    }
    print!("\n");
    // 创建广播服务器线程
    thread::spawn(move || {
        while let Ok(msg) = server.rx_from_threads.recv() {
            server.send_msg(msg);
        }
    });

    // 创建n个线程执行用户操作
    for _ in 0..n {
        let mut user = threads.pop().unwrap();
        thread::spawn(move || {

            
            let message = user.client.start();
            for msg in message.into_iter(){
                match msg {
                    Some(m) =>{ 
                        println!("Thread {} send message to {:?}\n{}", user.thread_id, m.receiver_id, m);
                        user.tx_to_server.send(m).unwrap()
                    }
                    None => (),
                }
            }

            while let Ok(msg) = user.rx_from_server.recv() {

                let new_msg = user.client.handle_message(msg);
                match new_msg {
                    Some(m) =>{ 
                        println!("Thread {} send message to {:?}\n{}", user.thread_id, m.receiver_id, m);
                        user.tx_to_server.send(m).unwrap()
                    }
                    None => (),
                }
            }
        });
    }

    // 等待所有线程完成
    thread::sleep(std::time::Duration::from_secs(5));
}
