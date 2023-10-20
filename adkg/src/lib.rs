pub mod server{
    pub mod servers;
}

pub mod client{
    pub mod gather;
    pub mod clients;
    pub mod vaba;
    pub mod adkg;
    pub mod avss;
}

pub mod msg{
    pub mod message;
    pub mod result;
}

use std::thread;
use std::sync::mpsc;
use std::collections::HashMap;

use crate::server::servers::{BroadcastServer, UserThread};
use crate::client::clients::Client;

/// 运行协议，参数 `n` 为参与方总数量，`f` 为恶意参与方数量
pub fn run(n: usize, f: usize) {
    // 创建通道，用于线程向服务器发送消息
    let (tx_to_server, rx_to_server) = mpsc::channel();

    let mut threads = Vec::new();
    let mut server = BroadcastServer {
        n,
        rx_from_threads: rx_to_server,
        tx_to_threads: HashMap::new(),
    };

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
        });
    }
    print!("\n");
    
    // 创建广播服务器线程
    thread::spawn(move || {
        while let Ok(msg) = server.rx_from_threads.recv() {
            server.send_msg(msg);
        }
    });

    let mut join_handles = Vec::new();

    // 创建 n 个线程执行用户操作
    for i in 0..n {
        let user = threads.pop().unwrap();
        join_handles.push(thread::spawn( move || {

            let mut user_node = Client::new(
                user.thread_id, 
                if i < n-f {1} else {0},
                n,
                f,
            );
            print!("thread id: {}, state: {}\n", user.thread_id, user_node.state);
            // 向服务器发送一条广播消息，开始协议
            let message = user_node.start();
            match message {
                Some(m) => user.tx_to_server.send(m).unwrap(),
                None => ()
            }
            
            // 等待服务器返回消息，处理消息，然后再向服务器发送消息
            while let Ok(msg) = user.rx_from_server.recv() {
                let new_msg = user_node.handle_message(msg);
                match new_msg {
                    Some(m) =>{ 
                        // println!("Thread {} send message to {:?}\n{}", user.thread_id, m.receiver_id, m);
                        user.tx_to_server.send(m).unwrap()
                    }
                    None => (),
                }
            }
            // println!("Thread {} finished", user.thread_id);
        }));
    }

    // 等待所有线程完成
    thread::sleep(std::time::Duration::from_secs(2));
    
}



#[cfg(test)]
mod tests {
    use super::run;
    use std::time::Instant;
    // use colored::*;
    // use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
    // use std::io::Write;
    #[test]
    fn t() {
        for n in (7..=12).step_by(4) {
            let f = (n-1)/3;
            println!("n: {}, f: {}", n, f);
            let start = Instant::now();
            run(n, f);
            let end = start.elapsed();
            println!("Time elapsed in run() is: {:?}", end);
        }
        
    }

    // #[test]
    // fn t2() {
    //     let mut stdout = StandardStream::stdout(ColorChoice::Always);
    //     let mut color = ColorSpec::new();
    //     color.set_fg(Some(Color::Red)).set_bold(true);
    //     for n in (7..=12).step_by(4) {
    //         let f = (n-1)/3;
    //         write!(&mut stdout, "n: {}, f: {}", n, f).unwrap();
    //         stdout.set_color(&color).unwrap();
    //         stdout.reset().unwrap();
    //         writeln!(&mut stdout).unwrap();

    //         let start = Instant::now();
    //         run(n, f);
    //         let end = start.elapsed();

    //         write!(&mut stdout, "Time elapsed in run() is: {:?}", end).unwrap();
    //         stdout.set_color(&color).unwrap();
    //         stdout.reset().unwrap();
    //         writeln!(&mut stdout).unwrap();
    //     }
    // }
}