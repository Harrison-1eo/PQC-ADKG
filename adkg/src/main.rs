use std::thread;
use std::sync::mpsc;
use std::collections::HashMap;

use multi_user::server::servers::{BroadcastServer, UserThread};
use multi_user::client::clients::Client;


/// 运行协议，参数 `n` 为参与方总数量，`f` 为恶意参与方数量
fn run(n: usize, f: usize) {
    // 创建通道，用于线程向服务器发送消息
    let (tx_to_server, rx_to_server) = mpsc::channel();

    let mut threads = Vec::new();
    let mut server = BroadcastServer {
        n,
        rx_from_threads: rx_to_server,
        tx_to_threads: HashMap::new(),
    };

    // 创建 n-f 个正常用户，这些用户的 thread_id 为 0..n-f
    // 创建 f 个恶意用户，这些用户的 thread_id 为 n-f..n
    for i in 0..n {
        print!(" {} ", i);
        let thread_id = i;

        let (tx_to_thread, rx_to_thread) = mpsc::channel();
        // thread_senders.insert(thread_id, tx_to_thread);
        server.tx_to_threads.insert(thread_id, tx_to_thread);

        threads.push(UserThread {
            thread_id,
            tx_to_server: tx_to_server.clone(),
            rx_from_server: rx_to_thread,
            client: Client::new(
                thread_id, 
                if i < n-f {1} else {0},
                n,
                f,
            )
        });
    }
    print!("\n");
    
    // 创建广播服务器线程
    thread::spawn(move || {
        while let Ok(msg) = server.rx_from_threads.recv() {
            server.send_msg(msg);
        }
    });

    // 创建 n 个线程执行用户操作
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
