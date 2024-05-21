use std::net::UdpSocket;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:5005")?;
    let counter = Arc::new(AtomicUsize::new(0));

    println!("Server listening on 127.0.0.1:5005");

    loop {
        let mut buf = [0u8; 1];
        let (_, src) = socket.recv_from(&mut buf)?;

        let amount = buf[0] as usize;
        let count = if amount > 0 {
            counter.fetch_add(amount, Ordering::SeqCst) + amount
        } else {
            counter.load(Ordering::SeqCst)
        };

        let response = count.to_be_bytes();
        socket.send_to(&response, &src)?;
        println!("Current counter is {}", count);
    }
}
