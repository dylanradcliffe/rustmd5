use std::{
    cmp::min,
    io::{self, BufRead, Read},
};
const BUFFER_SIZE_WORDS: usize = 16;
const BUFFER_WORD_SIZE: usize = 4;
const BUFFER_SIZE_BYTES: usize = BUFFER_SIZE_WORDS * BUFFER_WORD_SIZE;
type MD5WordBuffer = [u32; BUFFER_SIZE_WORDS];
type MD5ByteBuffer = [u8; BUFFER_SIZE_BYTES];

pub struct Block<'a> {
    data: MD5WordBuffer,
    reader: MD5Reader<'a>,
}

impl Block<'_> {
    fn new(r: MD5Reader) -> Block {
        Block {
            data: [0; BUFFER_SIZE_WORDS],
            reader: r,
        }
    }
}

enum MD5ReaderState {
    Reading,
    Padding(MD5ByteBuffer),
    Done,
}

pub struct MD5Reader<'a> {
    reader: &'a mut dyn Read,
    state: MD5ReaderState,
    length: usize,
}

impl MD5Reader<'_> {
    pub fn new(b: &mut impl Read) -> MD5Reader {
        MD5Reader {
            reader: b,
            state: MD5ReaderState::Reading,
            length: 0,
        }
    }
    pub fn readBlock(&mut self, buf: &mut MD5WordBuffer) -> io::Result<usize> {
        match self.state {
            MD5ReaderState::Reading => {
                let mut byte_buf: MD5ByteBuffer = [0; BUFFER_SIZE_BYTES];
                let mut count: usize = self.reader.read(&mut byte_buf)?;
                self.length += count;
                if count < BUFFER_SIZE_BYTES {
                    let pad = Self::padding(self.length, &mut byte_buf, count);
                }
                Self::bytes_to_words(&byte_buf, buf);
                Ok(count)
            }
            MD5ReaderState::Padding(pad_buf) => {
                Self::bytes_to_words(&pad_buf, buf);
                Ok(pad_buf.len())
            }
            MD5ReaderState::Done => Ok(0),
        }
    }

    fn padding(len: usize, buf: &mut MD5ByteBuffer, offset: usize) -> Option<MD5ByteBuffer> {
        let additonal_padding = (56 + 64 - ((len + 1) % 64)) % 64; // how many more pading bytes needed
        let mut fill_count = 0;
        let bts = len.to_le_bytes();

        let pad_value = |i: usize| -> u8 {
            if i == 0 {
                0b10000000
            } else if i < additonal_padding - 8 {
                0
            } else if i < additonal_padding - 8 + bts.len() {
                bts[i + 8 - additonal_padding]
            } else {
                0
            }
        };

        let fill_to = min(offset + additonal_padding, BUFFER_SIZE_BYTES);
        println!("{} {} {}", offset, additonal_padding, fill_to);
        for i in offset..fill_to {
            buf[i] = pad_value(fill_count);
            fill_count += 1;
        }

        if fill_to < offset + additonal_padding {
            let mut pbuf: MD5ByteBuffer = [0; BUFFER_SIZE_BYTES];
            for i in 0..(offset + additonal_padding - fill_to) {
                pbuf[i] = pad_value(fill_count);
                fill_count += 1;
            }
            Some(pbuf)
        } else {
            None
        }
    }

    fn bytes_to_words(bb: &MD5ByteBuffer, wb: &mut MD5WordBuffer) {
        for i in 0..wb.len() {
            let bbs = &bb[i * BUFFER_WORD_SIZE..(i + 1) * BUFFER_WORD_SIZE];
            wb[i] = ((bbs[0] as u32) << 0)
                | ((bbs[1] as u32) << 8)
                | ((bbs[2] as u32) << 16)
                | ((bbs[3] as u32) << 24)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::md5::{self, MD5Reader, MD5WordBuffer, BUFFER_SIZE_WORDS};

    #[test]
    fn it_works() {
        let mut binding = "ABCDEF".as_bytes();
        let mut reader = MD5Reader::new(&mut binding);
        let mut buf: MD5WordBuffer = [0; BUFFER_SIZE_WORDS];
        reader.readBlock(&mut buf);
        println!("{:x?}", buf)
    }
}
