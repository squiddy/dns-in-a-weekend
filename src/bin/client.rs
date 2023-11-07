use anyhow::Result;
use dns_in_a_weekend::constants::ROOT_SERVERS_V4;
use dns_in_a_weekend::constants::{Class, Flags, Type};
use dns_in_a_weekend::wire::{Header, Message, Question};
use std::net::{Ipv4Addr, UdpSocket};

/// Returns response after querying a server for a specific record. This
/// typically is not used directly.
///
/// # Arguments
///
/// * `destination` - IP address of dns server
/// * `domain_name` - Domain name to query
/// * `record_type` - E.g. MX, A, TXT
///
/// # Example
///
/// ```
/// send_query("8.8.8.8".parse()?, "www.example.com", Type::A);
/// ```
fn send_query(destination: Ipv4Addr, domain_name: &str, record_type: Type) -> Result<Message> {
    // TODO: find random port to bind socket to
    let socket = UdpSocket::bind("0.0.0.0:34254")?;

    let mut query = Message {
        header: Header {
            id: rand::random::<u16>(),
            flags: 0,
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
/// resolve("www.example.com", Type::A);
/// ```
fn resolve(domain_name: &str, record_type: Type) -> Result<Vec<u8>> {
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
            flags: Flags::RECURSION_DESIRED as u16,
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
