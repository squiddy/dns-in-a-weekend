use anyhow::Result;
use dns_in_a_weekend::constants::{Class, Flags, Type};
use dns_in_a_weekend::resolve;
use dns_in_a_weekend::wire::{Header, Message, Question};
use std::net::{Ipv4Addr, UdpSocket};

/// Return IPv4 address for given domain name based on the response from
/// Google's DNS servers.
///
/// # Arguments
///
/// * `domain_name` - Domain name to query
///
/// # Example
///
/// ```
/// lookup_domain("www.example.com");
/// ```
fn lookup_domain(domain_name: &str) -> Result<Ipv4Addr> {
    // TODO: find random port to bind socket to
    let socket = UdpSocket::bind("0.0.0.0:34254")?;

    let mut query = Message {
        header: Header {
            id: rand::random::<u16>(),
            flags: Flags::MESSAGE_REQUEST as u16 | Flags::RECURSION_DESIRED as u16,
            num_questions: 0,
            num_answers: 0,
            num_authorities: 0,
            num_additionals: 0,
        },
        questions: vec![Question {
            name: domain_name.to_string(),
            r#type: Type::A,
            class: Class::IN,
        }],
        answers: vec![],
        authorities: vec![],
        additionals: vec![],
    };

    let mut send_buffer = bytebuffer::ByteBuffer::new();
    query.write_to(&mut send_buffer);
    socket.send_to(send_buffer.as_bytes(), "8.8.8.8:53")?;

    let mut recv_buffer = [0; 512];
    let (amount, _) = socket.recv_from(&mut recv_buffer)?;

    let mut read_buffer = bytebuffer::ByteReader::from_bytes(&recv_buffer[..amount]);
    let message = Message::read_from(&mut read_buffer)?;

    let mut result = None;
    let mut query = domain_name.to_string();
    message
        .answers
        .iter()
        .for_each(|answer| match answer.r#type {
            Type::A if answer.name == query => {
                result = Some(&answer.data);
            }
            Type::CNAME => {
                query = String::from_utf8(answer.data.clone()).unwrap();
            }
            _ => (),
        });

    match result {
        Some(r) => Ok(Ipv4Addr::new(r[0], r[1], r[2], r[3])),
        None => Err(anyhow::anyhow!("Found no A record for {}", domain_name)),
    }
}

fn main() -> Result<()> {
    dbg!(lookup_domain("example.com")?);
    dbg!(lookup_domain("recurse.com")?);
    dbg!(lookup_domain("metafilter.com")?);
    dbg!(lookup_domain("www.metafilter.com")?);

    dbg!(resolve("example.com", Type::A)?);
    dbg!(resolve("recurse.com", Type::A)?);
    dbg!(resolve("metafilter.com", Type::A)?);
    dbg!(resolve("www.metafilter.com", Type::A)?);

    Ok(())
}
