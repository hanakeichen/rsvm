use super::ClassLoadErr;
use std::convert::TryInto;

pub trait ClassReader {
    fn read_ubyte1(&mut self) -> Result<u8, ClassLoadErr> {
        if self.offset() + 1 > self.class_bytes().len() {
            return Err(ClassLoadErr::InvalidFormat(
                "out of range, expected 1 byte".to_string(),
            ));
        }
        let result = self.class_bytes()[self.offset()];
        self.skip(1);
        Ok(result)
    }

    fn read_ubyte2(&mut self) -> Result<u16, ClassLoadErr> {
        if self.offset() + 2 > self.class_bytes().len() {
            return Err(ClassLoadErr::InvalidFormat(
                "out of range, expected 2 bytes".to_string(),
            ));
        }
        let bytes: &[u8] = &self.class_bytes()[self.offset()..self.offset() + 2];
        let bytes: [u8; 2] = bytes
            .try_into()
            .map_err(|_| ClassLoadErr::InvalidFormat("cannot read 2 bytes".to_string()))?;
        let result = u16::from_be_bytes(bytes);
        self.skip(2);
        Ok(result)
    }

    fn read_ubyte4(&mut self) -> Result<u32, ClassLoadErr> {
        if self.offset() + 4 > self.class_bytes().len() {
            return Err(ClassLoadErr::InvalidFormat(
                "out of range, expected 4 bytes".to_string(),
            ));
        }
        let bytes: &[u8] = &self.class_bytes()[self.offset()..self.offset() + 4];
        let bytes: [u8; 4] = bytes
            .try_into()
            .map_err(|_| ClassLoadErr::InvalidFormat("cannot read 4 bytes".to_string()))?;
        let result = u32::from_be_bytes(bytes);
        self.skip(4);
        Ok(result)
    }

    fn peek_nbytes(&mut self, n: usize) -> Result<&[u8], ClassLoadErr> {
        if self.offset() + n > self.class_bytes().len() {
            return Err(ClassLoadErr::InvalidFormat(format!(
                "out of range, expected {} bytes",
                n
            )));
        }
        let bytes: &[u8] = &self.class_bytes()[self.offset()..self.offset() + n];
        return Ok(bytes);
    }

    fn offset(&self) -> usize;

    fn readable_length(&self) -> usize {
        self.class_bytes().len()
    }

    fn buffer(&self) -> *const u8 {
        self.class_bytes().as_ptr()
    }

    fn available_buffer(&self) -> *const u8 {
        self.available_bytes().as_ptr()
    }

    fn skip(&mut self, size: usize);

    fn class_bytes(&self) -> &[u8];

    fn available_bytes(&self) -> &[u8] {
        return &self.class_bytes()[self.offset()..];
    }
}

pub struct OwnedBytesClassReader {
    class_bytes: Vec<u8>,
    offset: usize,
}

impl OwnedBytesClassReader {
    pub fn new(class_bytes: Vec<u8>) -> Self {
        OwnedBytesClassReader {
            class_bytes,
            offset: 0,
        }
    }
}

impl ClassReader for OwnedBytesClassReader {
    fn offset(&self) -> usize {
        self.offset
    }

    fn skip(&mut self, size: usize) {
        self.offset += size;
    }

    fn class_bytes(&self) -> &[u8] {
        self.class_bytes.as_slice()
    }
}

pub struct ExternalBytesClassReader<'a> {
    class_bytes: &'a [u8],
    offset: usize,
}

impl<'a> ExternalBytesClassReader<'a> {
    pub fn new(class_bytes: &'a [u8]) -> Self {
        ExternalBytesClassReader {
            class_bytes,
            offset: 0,
        }
    }
}

impl<'a> ClassReader for ExternalBytesClassReader<'a> {
    fn offset(&self) -> usize {
        self.offset
    }

    fn skip(&mut self, size: usize) {
        self.offset += size;
    }

    fn class_bytes(&self) -> &[u8] {
        self.class_bytes
    }
}
