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
        i = i % self.data.len() as isize;
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
        &self.data[self.modulo(self.begin as isize + index)]
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

const WINDOW_SIZE: usize = 0b1_0000_0000_0000;
const VIEW_SIZE: usize = 0b1_0000;

pub struct LZ77CodingIter<'a, I>
    where I: Iterator<Item = &'a u8>
{
    iter: I,
    window: Cycle<u8>,
    to_code: usize,
}

impl<'a, I> Iterator for LZ77CodingIter<'a, I>
    where I: Iterator<Item = &'a u8>
{
    type Item = (u16, u8, u8);
    fn next(&mut self) -> Option<(u16, u8, u8)> {
        while self.to_code < VIEW_SIZE {
            if let Some(&byte) = self.iter.next() {
                self.window.push(byte);
                self.to_code += 1;
            } else {
                break;
            }
        }

        // println!("{:?} to_code = {}", self.window, self.to_code);

        if self.to_code == 0 {
            return None;
        }

        // -----WINDOW-----|VIEW
        // 0

        let begin_view = -(self.to_code as isize);
        let mut len = 0;
        let mut ptr = None;

        'b: for j in 0..WINDOW_SIZE - self.to_code {
            if len + 1 == self.to_code as isize {
                break 'b;
            }
            for i in 0..len {
                if self.window.modulo(j as isize + i) == self.window.modulo(begin_view) {
                    break 'b;
                }
                if self.window[j as isize + i] != self.window[begin_view + i] {
                    continue 'b;
                }
            }
            while self.window.modulo(j as isize + len) != self.window.modulo(begin_view) &&
                  self.window[j as isize + len] == self.window[begin_view + len] {
                ptr = Some((WINDOW_SIZE - j - self.to_code) as u16);
                len += 1;
                if len + 1 == self.to_code as isize {
                    break 'b;
                }
            }
        }

        let k = self.window[begin_view + len];
        self.to_code -= len as usize + 1;

        // println!("=> (ptr = {:?}, len = {}, k = {})", ptr, len, k);

        Some((ptr.unwrap_or(0), len as u8, k))
    }
}

#[allow(dead_code)]
pub fn lz77_coding<'a, I>(iter: I) -> LZ77CodingIter<'a, I>
    where I: Iterator<Item = &'a u8>
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
        println!("Input {:?}", input);
        let coded: Vec<_> = lz77_coding(input.iter()).collect();
        println!("Coded {:?}", coded);
        let decoded: Vec<u8> = lz77_decoding(coded.iter()).collect();

        println!("Decoded {:?}", decoded);
        assert_eq!(input, decoded);
    }
    test(vec![]);
    test(vec![1]);
    test(vec![1, 1, 1, 1, 1, 1, 1]);
    test(vec![1, 2, 3, 1, 1, 2, 3, 1]);
    test(vec![1, 2, 3, 1, 1, 2, 3, 1, 1, 2, 3, 1, 1, 2, 3, 1, 2, 2, 1, 2, 3, 4, 5, 1, 2, 3, 4, 3,
              2, 1, 5, 4, 3, 4]);
    let mut xs = Vec::new();
    for i in 0..1024*128 {
        xs.push(((123 * i + 7) % 5) as u8);
    }
    // test(xs);
}
