use std::ops::Index;
use std::fmt::{Debug, Formatter, Error};

struct Cycle<T> {
    data: Vec<T>,
    begin: usize,
}

impl<T> Cycle<T>
    where T: Default + Clone
{
    fn new(size: usize) -> Cycle<T> {
        Cycle {
            data: vec![T::default(); size],
            begin: 0,
        }
    }

    fn push(&mut self, x: T) {
        self.data[self.begin] = x;
        self.begin = (self.begin + 1) % self.data.len();
    }

    fn modulo(&self, mut i: isize) -> usize {
        i = (self.begin as isize + i) % self.data.len() as isize;
        if i < 0 {
            i += self.data.len() as isize;
        }
        i as usize
    }
}

impl<T> Index<isize> for Cycle<T>
    where T: Default + Clone
{
    type Output = T;
    fn index(&self, index: isize) -> &T {
        &self.data[self.modulo(index)]
    }
}

impl<T> Debug for Cycle<T>
    where T: Default + Clone + Debug
{
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{:?}", &self.data[self.begin..])?;
        write!(f, "{:?}", &self.data[..self.begin])
    }
}

const WINDOW_SIZE: usize = 4096;
const VIEW_SIZE: usize = 32;

pub struct LZ77CodingIter<I>
    where I: Iterator<Item = u8>
{
    iter: I,
    window: Cycle<u8>,
    to_code: usize,
}

impl<I> Iterator for LZ77CodingIter<I>
    where I: Iterator<Item = u8>
{
    type Item = (u16, u8, u8);
    fn next(&mut self) -> Option<(u16, u8, u8)> {
        while self.to_code < VIEW_SIZE {
            if let Some(byte) = self.iter.next() {
                self.window.push(byte);
                self.to_code += 1;
            } else {
                break;
            }
        }

        if self.to_code == 0 {
            return None;
        }

        // -----WINDOW-----|VIEW
        // 0                 ^-- size to_code

        let mut len: usize = 0;
        let mut ptr: Option<u16> = None;

        if self.to_code > 1 {
            'b: for j in 0..WINDOW_SIZE - self.to_code {
                if j + len == WINDOW_SIZE - self.to_code {
                    break 'b;
                }
                for i in 0..len {
                    if self.window[j as isize + i as isize] !=
                       self.window[i as isize - self.to_code as isize] {
                        continue 'b;
                    }
                }
                while j + len < WINDOW_SIZE - self.to_code &&
                      self.window[j as isize + len as isize] ==
                      self.window[len as isize - self.to_code as isize] {
                    ptr = Some((WINDOW_SIZE - j - self.to_code) as u16);
                    len += 1;
                    if len + 1 == self.to_code {
                        break 'b;
                    }
                }
            }
        }

        let k = self.window[len as isize - self.to_code as isize];
        self.to_code -= len + 1;

        Some((ptr.unwrap_or(0), len as u8, k))
    }
}

#[allow(dead_code)]
pub fn lz77_coding<I>(iter: I) -> LZ77CodingIter<I>
    where I: Iterator<Item = u8>
{
    LZ77CodingIter {
        iter: iter,
        window: Cycle::new(WINDOW_SIZE),
        to_code: 0,
    }
}

/**************************************************************************************************
 decoding
*/


pub struct LZ77DecodingIter<'a, I>
    where I: Iterator<Item = &'a (u16, u8, u8)>
{
    iter: I,
    window: Cycle<u8>,
    decoded: usize,
}

impl<'a, I> Iterator for LZ77DecodingIter<'a, I>
    where I: Iterator<Item = &'a (u16, u8, u8)>
{
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        if self.decoded == 0 {
            if let Some(&(ptr, len, byte)) = self.iter.next() {
                for _ in 0..len {
                    let byte = self.window[WINDOW_SIZE as isize - ptr as isize];
                    self.window.push(byte);
                }
                self.window.push(byte);
                self.decoded += len as usize + 1;
            } else {
                return None;
            }
        }

        let result = self.window[-(self.decoded as isize)];
        self.decoded -= 1;
        Some(result)
    }
}

#[allow(dead_code)]
pub fn lz77_decoding<'a, I>(iter: I) -> LZ77DecodingIter<'a, I>
    where I: Iterator<Item = &'a (u16, u8, u8)>
{
    LZ77DecodingIter {
        iter: iter,
        window: Cycle::new(WINDOW_SIZE),
        decoded: 0,
    }
}

#[test]
fn lz_77_testing() {
    fn test(input: Vec<u8>) {
        // println!("Input {:?}", input);
        let coded: Vec<_> = lz77_coding(input.iter().cloned()).collect();
        // println!("Coded {:?}", coded);
        let decoded: Vec<u8> = lz77_decoding(coded.iter()).collect();

        // println!("Decoded {:?}", decoded);
        assert_eq!(input, decoded);
    }
    test(vec![]);
    test(vec![1]);
    test(vec![1, 1, 1, 1, 1, 1, 1]);
    test(vec![1, 2, 3, 1, 1, 2, 3, 1]);
    test(vec![1, 2, 3, 1, 1, 2, 3, 1, 1, 2, 3, 1, 1, 2, 3, 1, 2, 2, 1, 2, 3, 4, 5, 1, 2, 3, 4, 3,
              2, 1, 5, 4, 3, 4]);
    test(vec![91, 114, 111, 111, 116, 93, 10, 110, 97, 109, 101, 32, 61, 32, 34, 100, 97, 116,
              97, 95, 99, 111, 109, 112, 114, 101, 115, 115, 105, 111, 110, 34, 10, 118, 101,
              114, 115, 105, 111, 110, 32, 61, 32, 34, 48, 46, 49, 46, 48, 34, 10, 100, 101, 112,
              101, 110, 100, 101, 110, 99, 105, 101, 115, 32, 61, 32, 91, 10, 32, 34, 98, 105,
              110, 97, 114, 121, 95, 104, 101, 97, 112, 95, 99, 111, 109, 112, 97, 114, 101, 32,
              48, 46, 49, 46, 48, 32, 40, 103, 105, 116, 43, 104, 116, 116, 112, 115, 58, 47, 47,
              103, 105, 116, 104, 117, 98, 46, 99, 111, 109, 47, 97, 110, 116, 105, 103, 111,
              108, 47, 98, 105, 110, 97, 114, 121, 95, 104, 101, 97, 112, 95, 99, 111, 109, 112,
              97, 114, 101, 41, 34, 44, 10, 32, 34, 98, 105, 116, 45, 118, 101, 99, 32, 48, 46,
              52, 46, 52, 32, 40, 114, 101, 103, 105, 115, 116, 114, 121, 43, 104, 116, 116, 112,
              115, 58, 47, 47, 103, 105, 116, 104, 117, 98, 46, 99, 111, 109, 47, 114, 117, 115,
              116, 45, 108, 97, 110]);
    let mut xs = Vec::new();
    for i in 0..1024 * 128 {
        xs.push(((123 * i + 7) % 5) as u8);
    }
    test(xs);
}
