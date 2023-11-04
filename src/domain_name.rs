//! Handle decoding of (potentially) compressed domain names.
//! <https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.4>

use anyhow::Result;

pub fn decode_name(reader: &mut bytebuffer::ByteReader) -> Result<String> {
    let mut parts: Vec<String> = vec![];
    loop {
        let length = reader.read_u8()?;
        if length == 0 {
            break;
        }
        if length & 0b1100_0000 == 192 {
            parts.extend(decode_compressed_name(length, reader));
            break;
        } else {
            let part = reader.read_bytes(length as usize)?;
            let name = String::from_utf8(part)?;
            parts.push(name);
        }
    }

    Ok(parts.join("."))
}

fn decode_compressed_name(length: u8, reader: &mut bytebuffer::ByteReader) -> Result<String> {
    let offset = ((length & 0b0011_1111) as u16) << 8 | reader.read_u8()? as u16;
    let old_rpos = reader.get_rpos();
    reader.set_rpos(offset as usize);
    let result = decode_name(reader)?;
    reader.set_rpos(old_rpos);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use bytebuffer::ByteReader;

    use super::decode_name;

    #[test]
    fn decodes_name() {
        let mut buf = ByteReader::from_bytes(&[3, 65, 66, 67, 2, 68, 69, 0]);
        let result = decode_name(&mut buf);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "ABC.DE");
    }

    #[test]
    fn decodes_compressed_name() {
        let mut buf = ByteReader::from_bytes(&[3, 65, 66, 67, 0, 0, 0, 1, 65, 192, 0]);
        buf.set_rpos(7);
        let result = decode_name(&mut buf);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "A.ABC");
    }

    #[test]
    #[ignore]
    fn does_not_handle_loops_in_domain_name_compression() {
        let mut buf = ByteReader::from_bytes(&[192, 0]);
        // This overflows the stack
        let _ = decode_name(&mut buf);
    }
}
