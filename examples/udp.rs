#[derive(bin_proto::Protocol, Clone, Debug, PartialEq)]
pub struct Handshake;

#[derive(bin_proto::Protocol, Clone, Debug, PartialEq)]
pub struct Hello {
    id: i64,
    data: Vec<u8>
}

#[derive(bin_proto::Protocol, Clone, Debug, PartialEq)]
pub enum Packet {
    Handshake(Handshake),
    Hello(Hello),
}

fn main() {
    use std::net::UdpSocket;

    let socket = UdpSocket::bind("127.0.0.1:34254").unwrap();
    socket.connect("127.0.0.1:53111").unwrap();

    let settings = bin_proto::Settings::default();
    let mut pipeline = bin_proto::wire::dgram::Pipeline::new(bin_proto::wire::middleware::pipeline::default(), settings);


    // Send some data.
    {
        let mut send_packet = |packet| {
            let mut buffer = std::io::Cursor::new(Vec::new());
            pipeline.send_to(&mut buffer, &packet).unwrap();
            socket.send(&buffer.into_inner()).unwrap();
        };

        send_packet(Packet::Handshake(Handshake));
        send_packet(Packet::Hello(Hello { id: 51, data: vec![ 1, 2, 3]}));
    }

    loop {
        let mut buffer = [0u8; 10000];
        let bytes_read = socket.recv(&mut buffer).unwrap();
        let mut data = std::io::Cursor::new(&buffer[0..bytes_read]);

        let response = pipeline.receive_from(&mut data).unwrap();

        println!("{:?}", response);
        break;
    }
}

