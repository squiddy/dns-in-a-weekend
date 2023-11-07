///! A very, very rudimentary DNS server.
use anyhow::Result;
use dns_in_a_weekend::{
    constants::Flags,
    wire::{Header, Message},
};
use std::net::UdpSocket;

fn main() -> Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:53")?;
    let mut buf = [0; 512];
    loop {
        let (len, addr) = socket.recv_from(&mut buf)?;
        let mut reader = bytebuffer::ByteReader::from_bytes(&buf);
        match Message::read_from(&mut reader) {
            Ok(request) => {
                println!("{:?}", request);

                let mut writer = bytebuffer::ByteBuffer::new();
                let mut response = Message {
                    header: Header {
                        id: request.header.id,
                        flags: Flags::MESSAGE_RESPONSE as u16,
                        num_questions: request.header.num_questions,
                        num_answers: 0,
                        num_authorities: 0,
                        num_additionals: 0,
                    },
                    questions: request.questions,
                    answers: vec![],
                    authorities: vec![],
                    additionals: vec![],
                };
                response.write_to(&mut writer);
                // TODO: Tried giving bytebuffer an existing slice to avoid
                // allocations, but it wouldn't write the data.
                let send_buf = writer.into_vec();
                socket.send_to(&send_buf, addr)?;
            }
            Err(e) => eprintln!("{:?}", e),
        };
    }
}
