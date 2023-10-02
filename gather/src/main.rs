pub mod server;


fn main() {
    println!("Hello, world!");
    let mut server = server::servers::BroadcastServer::new(7, 2);
    server.start();
    server.broadcast2all();

}
