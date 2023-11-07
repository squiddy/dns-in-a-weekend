use crate::constants::{Class, Type};
use crate::domain_name::{decode_name, encode_name};
use anyhow::Result;

/// <https://datatracker.ietf.org/doc/html/rfc1035#section-4.1>
#[derive(Debug)]
pub struct Message {
    pub header: Header,
    pub questions: Vec<Question>,
    pub answers: Vec<ResourceRecord>,
    pub authorities: Vec<ResourceRecord>,
    pub additionals: Vec<ResourceRecord>,
}

impl Message {
    pub fn read_from(reader: &mut bytebuffer::ByteReader) -> Result<Self> {
        let header = Header::read_from(reader)?;
        let mut questions = vec![];
        let mut answers = vec![];
        let mut authorities = vec![];
        let mut additionals = vec![];

        for _ in 0..header.num_questions {
            questions.push(Question::read_from(reader)?);
        }

        for _ in 0..header.num_answers {
            answers.push(ResourceRecord::read_from(reader)?);
        }

        for _ in 0..header.num_authorities {
            authorities.push(ResourceRecord::read_from(reader)?);
        }

        for _ in 0..header.num_additionals {
            additionals.push(ResourceRecord::read_from(reader)?);
        }

        Ok(Self {
            header,
            questions,
            answers,
            authorities,
            additionals,
        })
    }

    pub fn write_to(&mut self, buffer: &mut bytebuffer::ByteBuffer) {
        self.header.num_questions = self.questions.len() as u16;
        self.header.num_answers = self.answers.len() as u16;
        self.header.num_authorities = self.authorities.len() as u16;
        self.header.num_additionals = self.additionals.len() as u16;
        self.header.write_to(buffer);

        for question in self.questions.iter_mut() {
            question.write_to(buffer);
        }
        for record in self
            .answers
            .iter_mut()
            .chain(self.authorities.iter_mut())
            .chain(self.additionals.iter_mut())
        {
            record.write_to(buffer);
        }
    }
}

/// <https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.3>
#[derive(Debug)]
pub struct ResourceRecord {
    pub name: String,
    pub r#type: Type,
    pub class: Class,
    pub ttl: u32,
    pub data: Vec<u8>,
}

impl ResourceRecord {
    fn read_from(reader: &mut bytebuffer::ByteReader) -> Result<Self>
    where
        Self: Sized,
    {
        let name = decode_name(reader)?;
        let r#type = reader.read_u16()?.try_into()?;
        let class = reader.read_u16()?.try_into()?;
        let ttl = reader.read_u32()?;
        let data_length = reader.read_u16()?;

        let data = match r#type {
            Type::CNAME | Type::NS => decode_name(reader)?.into_bytes(),
            _ => reader.read_bytes(data_length as usize)?,
        };

        Ok(Self {
            name,
            r#type,
            class,
            ttl,
            data,
        })
    }

    fn write_to(&mut self, buffer: &mut bytebuffer::ByteBuffer) {
        encode_name(buffer, &self.name);
        buffer.write_u16(self.r#type as u16);
        buffer.write_u16(self.class as u16);
        buffer.write_u32(self.ttl);
        buffer.write_u16(self.data.len() as u16);
        buffer.write_bytes(&self.data);
    }
}

/// <https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.1>
#[derive(Debug)]
pub struct Header {
    pub id: u16,
    pub flags: u16,
    pub num_questions: u16,
    pub num_answers: u16,
    pub num_authorities: u16,
    pub num_additionals: u16,
}

impl Header {
    fn write_to(&mut self, buffer: &mut bytebuffer::ByteBuffer) {
        buffer.write_u16(self.id);
        buffer.write_u16(self.flags);
        buffer.write_u16(self.num_questions);
        buffer.write_u16(self.num_answers);
        buffer.write_u16(self.num_authorities);
        buffer.write_u16(self.num_additionals);
    }

    fn read_from(reader: &mut bytebuffer::ByteReader) -> Result<Self> {
        Ok(Self {
            id: reader.read_u16()?,
            flags: reader.read_u16()?,
            num_questions: reader.read_u16()?,
            num_answers: reader.read_u16()?,
            num_authorities: reader.read_u16()?,
            num_additionals: reader.read_u16()?,
        })
    }
}

/// <https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.2>
#[derive(Debug)]
pub struct Question {
    pub name: String,
    pub r#type: Type,
    pub class: Class,
}

impl Question {
    fn read_from(reader: &mut bytebuffer::ByteReader) -> Result<Self> {
        Ok(Self {
            name: decode_name(reader)?,
            r#type: reader.read_u16()?.try_into()?,
            class: reader.read_u16()?.try_into()?,
        })
    }

    fn write_to(&mut self, buffer: &mut bytebuffer::ByteBuffer) {
        self.name.split('.').for_each(|part| {
            buffer.write_u8(part.len() as u8);
            buffer.write_bytes(part.as_bytes());
        });
        buffer.write_u8(0);
        buffer.write_u16(self.r#type as u16);
        buffer.write_u16(self.class as u16);
    }
}
