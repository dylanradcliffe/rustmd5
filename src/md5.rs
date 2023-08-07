//Implements https://www.ietf.org/rfc/rfc1321.txt

use std::{
    cmp::min,
    io::{self, Read},
    num::Wrapping,
};

const BUFFER_SIZE_WORDS: usize = 16;
const BUFFER_WORD_SIZE: usize = 4;
const BUFFER_SIZE_BYTES: usize = BUFFER_SIZE_WORDS * BUFFER_WORD_SIZE;
type MD5WordBuffer = [u32; BUFFER_SIZE_WORDS];
type MD5ByteBuffer = [u8; BUFFER_SIZE_BYTES];

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
    pub fn read_block(&mut self, buf: &mut MD5WordBuffer) -> io::Result<usize> {
        match self.state {
            MD5ReaderState::Reading => {
                let mut byte_buf: MD5ByteBuffer = [0; BUFFER_SIZE_BYTES];
                let count: usize = self.reader.read(&mut byte_buf)?;
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
        // last 8 bytes of 'padding' are length IN BITS and
        // at least one byte of padding must occur prior to this
        // the end and must align on a 64 byte block
        //  So 55 -> 64  (9 bytes "padding")
        //     56 -> 128 (72 byets "padding")
        let additonal_padding = (128 - 9 - (len % 64)) % 64 + 9;
        let mut fill_count = 0;
        let bts = (len * 8).to_le_bytes(); // *8 to get to bits

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
        //println!("offset={} ap={} ft={}", offset, additonal_padding, fill_to);
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

pub struct MD5Machine<'a> {
    block: MD5WordBuffer,
    reader: MD5Reader<'a>,
    a: u32,
    b: u32,
    c: u32,
    d: u32,
}

impl MD5Machine<'_> {
    pub fn new(r: MD5Reader) -> MD5Machine {
        MD5Machine {
            block: [0; BUFFER_SIZE_WORDS],
            reader: r,
            a: 0x67452301,
            b: 0xefcdab89,
            c: 0x98badcfe,
            d: 0x10325476,
        }
    }

    pub fn sum(&mut self) -> [u8; 16] {
        loop {
            let res = self.reader.read_block(&mut self.block);
            if let Ok(0) = res {
                break; // no more left
            }
            //println!("doing a round!");
            self.rounds() // run all the rounds
        }
        let mut output: [u8; 16] = [0; 16];
        output[0..4].copy_from_slice(&self.a.to_le_bytes());
        output[4..8].copy_from_slice(&self.b.to_le_bytes());
        output[8..12].copy_from_slice(&self.c.to_le_bytes());
        output[12..16].copy_from_slice(&self.d.to_le_bytes());

        output
    }

    fn rounds(&mut self) {
        let mut a: u32 = self.a;
        let mut b = self.b;
        let mut c = self.c;
        let mut d = self.d;
        let x = self.block;
        //println!("bar {:x} {:x} {:x} {:x}", self.a, self.b, self.c, self.d);
        // Below generated by python3 elementstransform.py elements.txt

        Self::op(&mut a, b, Self::aux_f(b, c, d), x[0], 7, 0xd76aa478); /* 1 */
        Self::op(&mut d, a, Self::aux_f(a, b, c), x[1], 12, 0xe8c7b756); /* 2 */
        Self::op(&mut c, d, Self::aux_f(d, a, b), x[2], 17, 0x242070db); /* 3 */
        Self::op(&mut b, c, Self::aux_f(c, d, a), x[3], 22, 0xc1bdceee); /* 4 */
        Self::op(&mut a, b, Self::aux_f(b, c, d), x[4], 7, 0xf57c0faf); /* 5 */
        Self::op(&mut d, a, Self::aux_f(a, b, c), x[5], 12, 0x4787c62a); /* 6 */
        Self::op(&mut c, d, Self::aux_f(d, a, b), x[6], 17, 0xa8304613); /* 7 */
        Self::op(&mut b, c, Self::aux_f(c, d, a), x[7], 22, 0xfd469501); /* 8 */
        Self::op(&mut a, b, Self::aux_f(b, c, d), x[8], 7, 0x698098d8); /* 9 */
        Self::op(&mut d, a, Self::aux_f(a, b, c), x[9], 12, 0x8b44f7af); /* 10 */
        Self::op(&mut c, d, Self::aux_f(d, a, b), x[10], 17, 0xffff5bb1); /* 11 */
        Self::op(&mut b, c, Self::aux_f(c, d, a), x[11], 22, 0x895cd7be); /* 12 */
        Self::op(&mut a, b, Self::aux_f(b, c, d), x[12], 7, 0x6b901122); /* 13 */
        Self::op(&mut d, a, Self::aux_f(a, b, c), x[13], 12, 0xfd987193); /* 14 */
        Self::op(&mut c, d, Self::aux_f(d, a, b), x[14], 17, 0xa679438e); /* 15 */
        Self::op(&mut b, c, Self::aux_f(c, d, a), x[15], 22, 0x49b40821); /* 16 */
        Self::op(&mut a, b, Self::aux_g(b, c, d), x[1], 5, 0xf61e2562); /* 17 */
        Self::op(&mut d, a, Self::aux_g(a, b, c), x[6], 9, 0xc040b340); /* 18 */
        Self::op(&mut c, d, Self::aux_g(d, a, b), x[11], 14, 0x265e5a51); /* 19 */
        Self::op(&mut b, c, Self::aux_g(c, d, a), x[0], 20, 0xe9b6c7aa); /* 20 */
        Self::op(&mut a, b, Self::aux_g(b, c, d), x[5], 5, 0xd62f105d); /* 21 */
        Self::op(&mut d, a, Self::aux_g(a, b, c), x[10], 9, 0x2441453); /* 22 */
        Self::op(&mut c, d, Self::aux_g(d, a, b), x[15], 14, 0xd8a1e681); /* 23 */
        Self::op(&mut b, c, Self::aux_g(c, d, a), x[4], 20, 0xe7d3fbc8); /* 24 */
        Self::op(&mut a, b, Self::aux_g(b, c, d), x[9], 5, 0x21e1cde6); /* 25 */
        Self::op(&mut d, a, Self::aux_g(a, b, c), x[14], 9, 0xc33707d6); /* 26 */
        Self::op(&mut c, d, Self::aux_g(d, a, b), x[3], 14, 0xf4d50d87); /* 27 */
        Self::op(&mut b, c, Self::aux_g(c, d, a), x[8], 20, 0x455a14ed); /* 28 */
        Self::op(&mut a, b, Self::aux_g(b, c, d), x[13], 5, 0xa9e3e905); /* 29 */
        Self::op(&mut d, a, Self::aux_g(a, b, c), x[2], 9, 0xfcefa3f8); /* 30 */
        Self::op(&mut c, d, Self::aux_g(d, a, b), x[7], 14, 0x676f02d9); /* 31 */
        Self::op(&mut b, c, Self::aux_g(c, d, a), x[12], 20, 0x8d2a4c8a); /* 32 */
        Self::op(&mut a, b, Self::aux_h(b, c, d), x[5], 4, 0xfffa3942); /* 33 */
        Self::op(&mut d, a, Self::aux_h(a, b, c), x[8], 11, 0x8771f681); /* 34 */
        Self::op(&mut c, d, Self::aux_h(d, a, b), x[11], 16, 0x6d9d6122); /* 35 */
        Self::op(&mut b, c, Self::aux_h(c, d, a), x[14], 23, 0xfde5380c); /* 36 */
        Self::op(&mut a, b, Self::aux_h(b, c, d), x[1], 4, 0xa4beea44); /* 37 */
        Self::op(&mut d, a, Self::aux_h(a, b, c), x[4], 11, 0x4bdecfa9); /* 38 */
        Self::op(&mut c, d, Self::aux_h(d, a, b), x[7], 16, 0xf6bb4b60); /* 39 */
        Self::op(&mut b, c, Self::aux_h(c, d, a), x[10], 23, 0xbebfbc70); /* 40 */
        Self::op(&mut a, b, Self::aux_h(b, c, d), x[13], 4, 0x289b7ec6); /* 41 */
        Self::op(&mut d, a, Self::aux_h(a, b, c), x[0], 11, 0xeaa127fa); /* 42 */
        Self::op(&mut c, d, Self::aux_h(d, a, b), x[3], 16, 0xd4ef3085); /* 43 */
        Self::op(&mut b, c, Self::aux_h(c, d, a), x[6], 23, 0x4881d05); /* 44 */
        Self::op(&mut a, b, Self::aux_h(b, c, d), x[9], 4, 0xd9d4d039); /* 45 */
        Self::op(&mut d, a, Self::aux_h(a, b, c), x[12], 11, 0xe6db99e5); /* 46 */
        Self::op(&mut c, d, Self::aux_h(d, a, b), x[15], 16, 0x1fa27cf8); /* 47 */
        Self::op(&mut b, c, Self::aux_h(c, d, a), x[2], 23, 0xc4ac5665); /* 48 */
        Self::op(&mut a, b, Self::aux_i(b, c, d), x[0], 6, 0xf4292244); /* 49 */
        Self::op(&mut d, a, Self::aux_i(a, b, c), x[7], 10, 0x432aff97); /* 50 */
        Self::op(&mut c, d, Self::aux_i(d, a, b), x[14], 15, 0xab9423a7); /* 51 */
        Self::op(&mut b, c, Self::aux_i(c, d, a), x[5], 21, 0xfc93a039); /* 52 */
        Self::op(&mut a, b, Self::aux_i(b, c, d), x[12], 6, 0x655b59c3); /* 53 */
        Self::op(&mut d, a, Self::aux_i(a, b, c), x[3], 10, 0x8f0ccc92); /* 54 */
        Self::op(&mut c, d, Self::aux_i(d, a, b), x[10], 15, 0xffeff47d); /* 55 */
        Self::op(&mut b, c, Self::aux_i(c, d, a), x[1], 21, 0x85845dd1); /* 56 */
        Self::op(&mut a, b, Self::aux_i(b, c, d), x[8], 6, 0x6fa87e4f); /* 57 */
        Self::op(&mut d, a, Self::aux_i(a, b, c), x[15], 10, 0xfe2ce6e0); /* 58 */
        Self::op(&mut c, d, Self::aux_i(d, a, b), x[6], 15, 0xa3014314); /* 59 */
        Self::op(&mut b, c, Self::aux_i(c, d, a), x[13], 21, 0x4e0811a1); /* 60 */
        Self::op(&mut a, b, Self::aux_i(b, c, d), x[4], 6, 0xf7537e82); /* 61 */
        Self::op(&mut d, a, Self::aux_i(a, b, c), x[11], 10, 0xbd3af235); /* 62 */
        Self::op(&mut c, d, Self::aux_i(d, a, b), x[2], 15, 0x2ad7d2bb); /* 63 */
        Self::op(&mut b, c, Self::aux_i(c, d, a), x[9], 21, 0xeb86d391); /* 64 */
        // End generated

        self.a = (Wrapping(a) + Wrapping(self.a)).0;
        self.b = (Wrapping(b) + Wrapping(self.b)).0;
        self.c = (Wrapping(c) + Wrapping(self.c)).0;
        self.d = (Wrapping(d) + Wrapping(self.d)).0;

        //println!("baz {:x} {:x} {:x} {:x}", self.a, self.b, self.c, self.d);
    }

    // Auxilary functions

    fn aux_f(x: u32, y: u32, z: u32) -> u32 {
        // F(X,Y,Z) = XY v not(X) Z
        (x & y) | ((!x) & z)
    }

    fn aux_g(x: u32, y: u32, z: u32) -> u32 {
        // G(X,Y,Z) = XZ v Y not(Z)
        (x & z) | (y & (!z))
    }

    fn aux_h(x: u32, y: u32, z: u32) -> u32 {
        // H(X,Y,Z) = X xor Y xor Z
        x ^ y ^ z
    }

    fn aux_i(x: u32, y: u32, z: u32) -> u32 {
        // I(X,Y,Z) = Y xor (X v not(Z))
        y ^ (x | (!z))
    }

    /* Let [abcd k s i] denote the operation
    a = b + ((a + F(b,c,d) + X[k] + T[i]) <<< s). */

    fn op(a: &mut u32, b: u32, f_bcd: u32, x_k: u32, s: u8, t_i: u32) {
        let unrotated = (Wrapping(*a) + Wrapping(f_bcd) + Wrapping(x_k) + Wrapping(t_i)).0;
        *a = (Wrapping(b) + Wrapping((unrotated << s) | (unrotated >> (32 - s)))).0;
    }
}
#[cfg(test)]
mod tests {
    use std::io::Read;

    use crate::md5::{MD5Reader, MD5WordBuffer, BUFFER_SIZE_BYTES, BUFFER_SIZE_WORDS};

    use super::MD5Machine;

    fn padding_tst(msg: &mut impl Read, expected: Vec<MD5WordBuffer>) {
        let mut reader = MD5Reader::new(msg);
        let mut buf: MD5WordBuffer = [0; BUFFER_SIZE_WORDS];

        for exp in expected {
            let res = reader.read_block(&mut buf);
            println!("buf={:x?}", buf);
            println!("exp={:x?}", exp);
            println!("res={:x?}", res);
            assert!(match res {
                Ok(BUFFER_SIZE_BYTES) => true,
                _ => false,
            });
            assert!(buf == exp);
        }
        let res = reader.read_block(&mut buf);
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
                0x44434241, 0x00804645, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x30, 0,
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
        let binding = [0x44; 55];
        padding_tst(
            &mut binding.as_ref(),
            vec![[
                0x44444444, 0x44444444, 0x44444444, 0x44444444, 0x44444444, 0x44444444, 0x44444444,
                0x44444444, 0x44444444, 0x44444444, 0x44444444, 0x44444444, 0x44444444, 0x80444444,
                0x000001B8, 0x00,
            ]],
        )
    }

    #[test]
    fn padding_equal() {
        //        let mut binding = "DDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD".as_bytes();
        let binding = [0x44; 56]; // should pad anyway
        padding_tst(
            &mut binding.as_ref(),
            vec![
                [
                    0x44444444, 0x44444444, 0x44444444, 0x44444444, 0x44444444, 0x44444444,
                    0x44444444, 0x44444444, 0x44444444, 0x44444444, 0x44444444, 0x44444444,
                    0x44444444, 0x44444444, 0x00000080, 0x00,
                ],
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x000001C0, 0x00],
            ],
        )
    }

    #[test]
    fn padding_over() {
        //        let mut binding = "DDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD".as_bytes();
        let binding = [0x44; 57]; // should pad anyway
        padding_tst(
            &mut binding.as_ref(),
            vec![
                [
                    0x44444444, 0x44444444, 0x44444444, 0x44444444, 0x44444444, 0x44444444,
                    0x44444444, 0x44444444, 0x44444444, 0x44444444, 0x44444444, 0x44444444,
                    0x44444444, 0x44444444, 0x00008044, 0x00,
                ],
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x000001C8, 0x00],
            ],
        )
    }

    fn md5reader_tst_str(msg: &str, exp: [u8; 16]) {
        let mut binding = msg.as_bytes();

        let reader: MD5Reader<'_> = MD5Reader::new(&mut binding);
        let mut machine = MD5Machine::new(reader);
        let sum = machine.sum();
        println!("got sum {:x?}", sum);
        assert!(sum == exp)
    }

    #[test]
    fn test_sums() {
        let msgs: [(&str, u128); 7] = [
            ("", 0xd41d8cd98f00b204e9800998ecf8427e),
            ("a", 0x0cc175b9c0f1b6a831c399e269772661),
            ("abc", 0x900150983cd24fb0d6963f7d28e17f72),
            ("message digest", 0xf96b697d7cb7938d525a2f31aaf161d0),
            (
                "abcdefghijklmnopqrstuvwxyz",
                0xc3fcd3d76192e4007dfb496cca67e13b,
            ),
            (
                "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789",
                0xd174ab98d277d9f5a5611c2c9f419d9f,
            ),
            (
                "12345678901234567890123456789012345678901234567890123456789012345678901234567890",
                0x57edf4a22be3c955ac49da2e2107b67a,
            ),
        ];

        for (msg, exp_b) in msgs {
            md5reader_tst_str(msg, exp_b.to_be_bytes())
        }
    }
}
