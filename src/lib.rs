//#![allow(dead_code)]
extern crate binary_heap_compare;
extern crate bit_vec;
use bit_vec::BitVec;
mod huffman;
use huffman::*;
mod bitvec_util;
mod lz_78;
use bitvec_util::*;


pub fn compression(content: Vec<u8>) -> Vec<u8> {
    let mut statistique = [0; 256];
    for &item in content.iter() {
        statistique[item as usize] += 1;
    }

    let tree = Node::from_statistics_u8(&statistique);

    let mut output = tree.encode_tree();


    let mut dictionnary = vec![BitVec::new();256];
    let root_word = BitVec::new();

    scan_tree(tree, root_word, &mut dictionnary);

    for &item in content.iter() {
        output = append_bit_vec(output, &dictionnary[item as usize]);
    }


    serialize_bit_vec(&output)
}

pub fn decompression(content: Vec<u8>) -> Vec<u8> {
    let input = deserialize_bit_vec(&content);
    let mut iter = input.iter();

    let tree = Node::decode_tree(&mut iter);

    let mut output: Vec<u8> = Vec::new();
    let mut reading_head = &tree;
    for bit in iter {
        if let &Node::Branch(ref node_0, ref node_1) = reading_head {
            if bit {
                reading_head = &node_1;
            } else {
                reading_head = &node_0;
            }
            if let &Node::Leaf(character) = reading_head {
                output.push(character);
                reading_head = &tree;
            }
        }
    }
    return output;
}
