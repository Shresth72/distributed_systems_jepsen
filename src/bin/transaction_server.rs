use std::net::UdpSocket;

fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:5005")?;

    println!("Server listening on 127.0.0.1:5005");

    loop {
        let mut buf = [0u8; 1];
        let (_, src) = socket.recv_from(&mut buf)?;

        let response = [0u8; 8];

        socket.send_to(&response, &src)?;
    }
}
