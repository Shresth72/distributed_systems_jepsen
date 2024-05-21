use std::net::UdpSocket;
use std::sync::atomic::{AtomicUsize, Ordering};

fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:5005")?;
    let counter = AtomicUsize::new(0);

    println!("Server listening on 127.0.0.1:5005");

    loop {
        let mut buf = [0u8; 1];
        let (_, src) = socket.recv_from(&mut buf)?;

        let amount = buf[0] as usize;

        let count = if amount > 0 {
            counter.fetch_add(amount, Ordering::SeqCst) + amount

            // Save into file (not including currently as Lampart Diag are already created in the store)
            // tokio::spawn(store_count(node_id, count));
        } else {
            counter.load(Ordering::SeqCst)
        };

        let response = count.to_be_bytes();
        socket.send_to(&response, &src)?;
    }
}

// async fn store_count(node_id: usize, count: usize) -> std::io::Result<()> {
//     let mut file = OpenOptions::new()
//         .create(true)
//         .write(true)
//         .truncate(true)
//         .open("count_data.txt")?;
//
//     file.seek(SeekFrom::End(0))?;
//     writeln!(file, "Node {}: {}", node_id, count)?;
//
//     Ok(())
// }
