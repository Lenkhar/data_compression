
extern crate huffman;
use std::fs::File;
use std::io::prelude::*;
use std::env;

fn main() {
    let file_name = env::args().nth(1).unwrap();

    let mut file = File::open(&file_name).unwrap();
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).unwrap();
    let output = huffman::compression(contents);

    let mut file = File::create(file_name + ".lm").unwrap();
    file.write_all(&output).unwrap();
}
