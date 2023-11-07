pub mod constants;
mod domain_name;
pub mod wire;

use anyhow::Result;
use constants::{Class, Flags, Type, ROOT_SERVERS_V4};
use std::net::{Ipv4Addr, UdpSocket};
use wire::{Header, Message, Question};

/// Returns response after querying a server for a specific record. This
/// typically is not used directly.
///
/// # Arguments
///
/// * `destination` - IP address of dns server
/// * `domain_name` - Domain name to query
/// * `record_type` - E.g. MX, A, TXT
fn send_query(destination: Ipv4Addr, domain_name: &str, record_type: Type) -> Result<Message> {
    // TODO: find random port to bind socket to
    let socket = UdpSocket::bind("0.0.0.0:34254")?;

    let mut query = Message {
        header: Header {
            id: rand::random::<u16>(),
            flags: Flags::MESSAGE_REQUEST as u16,
            num_questions: 0,
            num_answers: 0,
            num_authorities: 0,
            num_additionals: 0,
        },
        questions: vec![Question {
            name: domain_name.to_string(),
            r#type: record_type,
            class: Class::IN,
        }],
        answers: vec![],
        authorities: vec![],
        additionals: vec![],
    };

    let mut send_buffer = bytebuffer::ByteBuffer::new();
    query.write_to(&mut send_buffer);
    socket.send_to(send_buffer.as_bytes(), (destination, 53))?;

    let mut recv_buffer = [0; 512];
    let (amount, _) = socket.recv_from(&mut recv_buffer)?;

    let mut read_buffer = bytebuffer::ByteReader::from_bytes(&recv_buffer[..amount]);
    Message::read_from(&mut read_buffer)
}

/// Return record for a given domain.
///
/// This goes through the root DNS servers to find the record.
///
/// # Arguments
///
/// * `domain_name` - Domain name to query
/// * `record_type` - E.g. MX, A, TXT
///
/// # Example
///
/// ```
/// use dns_in_a_weekend::resolve;
/// use dns_in_a_weekend::constants::Type;
/// resolve("www.example.com", Type::A);
/// ```
pub fn resolve(domain_name: &str, record_type: Type) -> Result<Vec<u8>> {
    let mut nameserver = ROOT_SERVERS_V4[0].parse()?;

    loop {
        let message = send_query(nameserver, domain_name, record_type)?;
        let answer = message
            .answers
            .iter()
            .filter(|a| a.r#type == record_type)
            .map(|a| &a.data)
            .next();
        let authority = message
            .authorities
            .iter()
            .filter(|a| matches!(a.r#type, Type::NS))
            .map(|a| String::from_utf8(a.data.clone()).unwrap())
            .next();
        let additional = message
            .additionals
            .iter()
            .filter(|a| matches!(a.r#type, Type::A))
            .map(|a| Ipv4Addr::new(a.data[0], a.data[1], a.data[2], a.data[3]))
            .next();

        if let Some(ip) = answer {
            return Ok(ip.clone());
        } else if let Some(ns_ip) = additional {
            nameserver = ns_ip;
        } else if let Some(ns) = authority {
            let data = resolve(&ns, Type::A)?;
            nameserver = Ipv4Addr::new(data[0], data[1], data[2], data[3]);
        } else {
            break;
        }
    }

    Err(anyhow::anyhow!(
        "Couldn't resolve type:{:?} for {}",
        record_type,
        domain_name
    ))
}
