
extern crate data_compression;
use std::fs::File;
use std::io::prelude::*;
use std::env;

fn main() {
    let file_name = env::args().nth(1).unwrap();

    let mut file = File::open(&file_name).unwrap();
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).unwrap();
    let output = data_compression::decompression(contents).unwrap();

    let mut file = File::create(file_name.split_at(file_name.len() - 3).0).unwrap();
    file.write_all(&output).unwrap();
}
