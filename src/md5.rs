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
                    if let Some(pb) = pad {
                        self.state = MD5ReaderState::Padding(pb);
                    } else {
                        self.state = MD5ReaderState::Done;
                    }
                }
                Self::bytes_to_words(&byte_buf, buf);
                Ok(BUFFER_SIZE_BYTES)
            }
            MD5ReaderState::Padding(pad_buf) => {
                Self::bytes_to_words(&pad_buf, buf);
                self.state = MD5ReaderState::Done;
                Ok(pad_buf.len())
            }
            MD5ReaderState::Done => Ok(0),
        }
    }

    fn padding(len: usize, buf: &mut MD5ByteBuffer, offset: usize) -> Option<MD5ByteBuffer> {
        // last 8 bytes of 'padding' are length and
        // at least one byte of padding must occur prior to this
        // the end and must align on a 64 byte block
        //  So 55 -> 64  (9 bytes "padding")
        //     56 -> 128 (72 byets "padding")
        let additonal_padding = (128 - 9 - (len % 64)) % 64 + 9;
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
        println!("offset={} ap={} ft={}", offset, additonal_padding, fill_to);
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
    use std::io::Read;

    use crate::md5::{self, MD5Reader, MD5WordBuffer, BUFFER_SIZE_BYTES, BUFFER_SIZE_WORDS};

    fn padding_tst(msg: &mut impl Read, expected: Vec<MD5WordBuffer>) {
        let mut reader = MD5Reader::new(msg);
        let mut buf: MD5WordBuffer = [0; BUFFER_SIZE_WORDS];

        for exp in expected {
            let res = reader.readBlock(&mut buf);
            println!("buf={:x?}", buf);
            println!("exp={:x?}", exp);
            println!("res={:x?}", res);
            assert!(match res {
                Ok(BUFFER_SIZE_BYTES) => true,
                _ => false,
            });
            assert!(buf == exp);
        }
        let res = reader.readBlock(&mut buf);
        println!("buf={:x?}", buf);
        println!("res={:x?}", res);
        /*  assert!(match res {
            Ok(0) => true,
            _ => false,
        });*/
    }

    #[test]
    fn padding_simple() {
        let mut binding = "ABCDEF".as_bytes();
        padding_tst(
            &mut binding,
            vec![[
                0x44434241, 0x00804645, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x06, 0,
            ]],
        )
    }

    #[test]
    fn padding_empty() {
        let mut binding = "".as_bytes();
        padding_tst(
            &mut binding,
            vec![[0x00000080, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]],
        )
    }

    #[test]
    fn padding_close() {
        //        let mut binding = "DDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD".as_bytes();
        let mut binding = [0x44; 55];
        padding_tst(
            &mut binding.as_ref(),
            vec![[
                0x44444444, 0x44444444, 0x44444444, 0x44444444, 0x44444444, 0x44444444, 0x44444444,
                0x44444444, 0x44444444, 0x44444444, 0x44444444, 0x44444444, 0x44444444, 0x80444444,
                0x00000037, 0x00,
            ]],
        )
    }

    #[test]
    fn padding_equal() {
        //        let mut binding = "DDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD".as_bytes();
        let mut binding = [0x44; 56]; // should pad anyway
        padding_tst(
            &mut binding.as_ref(),
            vec![
                [
                    0x44444444, 0x44444444, 0x44444444, 0x44444444, 0x44444444, 0x44444444,
                    0x44444444, 0x44444444, 0x44444444, 0x44444444, 0x44444444, 0x44444444,
                    0x44444444, 0x44444444, 0x00000080, 0x00,
                ],
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x00000038, 0x00],
            ],
        )
    }

    #[test]
    fn padding_over() {
        //        let mut binding = "DDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD".as_bytes();
        let mut binding = [0x44; 57]; // should pad anyway
        padding_tst(
            &mut binding.as_ref(),
            vec![
                [
                    0x44444444, 0x44444444, 0x44444444, 0x44444444, 0x44444444, 0x44444444,
                    0x44444444, 0x44444444, 0x44444444, 0x44444444, 0x44444444, 0x44444444,
                    0x44444444, 0x44444444, 0x00008044, 0x00,
                ],
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x00000039, 0x00],
            ],
        )
    }
}
