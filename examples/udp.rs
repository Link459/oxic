use core::str;
use std::time::Duration;

use oxic::{net::udp::UdpSocket, prelude::Runtime};

fn main() {
    std::thread::spawn(move || {
        std::thread::sleep(Duration::new(1, 0));
        let sock2 = std::net::UdpSocket::bind("127.0.0.1:3005").unwrap();
        let n = sock2.send_to("hello".as_bytes(), "127.0.0.1:3004").unwrap();
        assert_eq!(n, 5);
    });

    let mut rt = Runtime::new();
    rt.block_on(async {
        let sock = UdpSocket::bind("127.0.0.1:3004").unwrap();
        let mut buf = [0; 5];
        let n = sock.recv(&mut buf).await.unwrap();
        println!("{}", str::from_utf8(&buf).unwrap());
        assert_eq!(n, 5);
        assert_eq!(buf, "hello".as_bytes());
    })
}
