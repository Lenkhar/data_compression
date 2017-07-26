extern crate binary_heap_compare;
extern crate bit_vec;
use bit_vec::BitVec;
mod huffman;
use huffman::*;
mod bitvec_util;
use bitvec_util::*;
use std::collections::BTreeMap;


pub fn compression(content: Vec<u8>) -> Vec<u8> {
    let mut statistique: BTreeMap<u8, u64> = BTreeMap::new();
    for item in content.iter() {
        if statistique.contains_key(item){
            *statistique.get_mut(item).unwrap() += 1;
        } else {
            statistique.insert(*item,1);
        }
    }

    let tree = Node::from_statistics(&statistique);

    let mut output = tree.encode_tree_u8();

    let dictionnary = tree.to_dictionnary(BitVec::new());

    for &character in content.iter() {
        output = append_bit_vec(output, &dictionnary[&character]);
    }

    serialize_bit_vec(&output)
}

pub fn decompression(content: Vec<u8>) -> Result<Vec<u8>, &'static str> {
    let input = deserialize_bit_vec(&content);
    let mut iter = input.iter();

    let tree = Node::decode_tree_u8(&mut iter)?;

    let mut output: Vec<u8> = Vec::new();
    while let Some(code) = tree.scan(&mut iter)? {
        output.push(code);
    }

    return Ok(output);
}
