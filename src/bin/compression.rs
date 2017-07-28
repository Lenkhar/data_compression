extern crate data_compression;
use std::fs::File;
use std::io::prelude::*;
use std::env;

fn main() {
    let file_name = env::args().nth(1).unwrap();

    let file = File::open(&file_name).unwrap();
    let iter = file.bytes().map(|x| x.unwrap());
    let output = data_compression::compression_lz77(iter);

    let mut file = File::create(file_name + ".lm").unwrap();
    file.write_all(&output).unwrap();
}
