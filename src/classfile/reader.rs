use super::ClassLoadErr;
use std::convert::TryInto;

pub trait ClassReader {
    fn read_ubyte1(&mut self) -> Result<u8, ClassLoadErr> {
        if self.offset() + 1 > self.class_bytes().len() {
            return Err("out of range");
        }
        let result = self.class_bytes()[self.offset()];
        self.skip(1);
        Ok(result)
    }

    fn read_ubyte2(&mut self) -> Result<u16, ClassLoadErr> {
        if self.offset() + 2 > self.class_bytes().len() {
            return Err("out of range");
        }
        let bytes: &[u8] = &self.class_bytes()[self.offset()..self.offset() + 2];
        let bytes: [u8; 2] = bytes.try_into().map_err(|_| "cannot read 2 bytes")?;
        let result = u16::from_be_bytes(bytes);
        self.skip(2);
        Ok(result)
    }

    fn read_ubyte4(&mut self) -> Result<u32, ClassLoadErr> {
        if self.offset() + 4 > self.class_bytes().len() {
            return Err("out of range");
        }
        let bytes: &[u8] = &self.class_bytes()[self.offset()..self.offset() + 4];
        let bytes: [u8; 4] = bytes.try_into().map_err(|_| "cannot read 4 bytes")?;
        let result = u32::from_be_bytes(bytes);
        self.skip(4);
        Ok(result)
    }

    fn offset(&self) -> usize;

    fn readable_length(&self) -> usize {
        self.class_bytes().len()
    }

    fn buffer(&self) -> *const u8 {
        self.class_bytes().as_ptr()
    }

    fn skip(&mut self, size: usize);

    fn class_bytes(&self) -> &[u8];
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
    fn new(class_bytes: &'a [u8]) -> Self {
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
