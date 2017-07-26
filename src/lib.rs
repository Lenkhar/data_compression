extern crate binary_heap_compare;
extern crate bit_vec;
use bit_vec::BitVec;
mod huffman;
use huffman::*;
mod bitvec_util;
mod lz_78;
use lz_78::{lz78_coding,lz78_decoding};
use bitvec_util::*;
use std::collections::BTreeMap;


pub fn compression(content: Vec<u8>) -> Vec<u8> {
    let lz78_coded : Vec<(u64,u8)> = lz78_coding(content.iter());

    let (mut pointer_statistic, mut character_statistic) : (BTreeMap<u64,u64>,BTreeMap<u8,u64>) = (BTreeMap::new(), BTreeMap::new());

    for item in lz78_coded.iter(){
        *pointer_statistic.entry(item.0).or_insert(0) += 1;
        *character_statistic.entry(item.1).or_insert(0) += 1;
    }

    let (pointer_tree, character_tree) = (Node::from_statistics(&pointer_statistic),Node::from_statistics(&character_statistic));

    let mut output = BitVec::new();
    output = append_bit_vec(output, &Node::encode_tree(&pointer_tree));
    output = append_bit_vec(output, &Node::encode_tree(&character_tree));

    let (pointer_dictionnary, character_dictionnary) = (pointer_tree.to_dictionnary(BitVec::new()), character_tree.to_dictionnary(BitVec::new()));
    for &(pointer, character) in lz78_coded.iter(){
        output=append_bit_vec(output,&pointer_dictionnary[&pointer]);
        output = append_bit_vec(output,&character_dictionnary[&character]);
    }

    serialize_bit_vec(&output)
}

pub fn decompression(content: Vec<u8>) -> Result<Vec<u8>, &'static str> {
    let input = deserialize_bit_vec(&content);
    let mut iter = input.iter();

    let (pointer_tree, character_tree) = (Node::decode_tree(&mut iter)?, Node::decode_tree(&mut iter)?);

    let mut lz78_coded: Vec<(u64,u8)> = Vec::new();

    while let Some(pointer_code) = pointer_tree.scan(&mut iter)? {
        while let Some(character_code) = character_tree.scan(&mut iter)?{
            lz78_coded.push((pointer_code,character_code));
        }
    }

    return lz78_decoding(lz78_coded.iter());
}
