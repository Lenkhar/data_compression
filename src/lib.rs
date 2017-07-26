extern crate binary_heap_compare;
extern crate bit_vec;
use bit_vec::BitVec;
mod huffman;
use huffman::*;
mod bitvec_util;
mod lz_78;
use lz_78::{lz78_coding, lz78_decoding};
use bitvec_util::*;
use std::collections::BTreeMap;


pub fn compression(content: &Vec<u8>) -> Vec<u8> {
    if content.is_empty() {
        return content.clone();
    }
    let lz78_coded: Vec<(u64, u8)> = lz78_coding(content.iter());

    let (mut pointer_statistic, mut character_statistic): (BTreeMap<u64, u64>,
                                                           BTreeMap<u8, u64>) = (BTreeMap::new(),
                                                                                 BTreeMap::new());

    for &(pointer, character) in lz78_coded.iter() {
        *pointer_statistic.entry(pointer).or_insert(0) += 1;
        *character_statistic.entry(character).or_insert(0) += 1;
    }

    let (pointer_tree, character_tree) = (Node::from_statistics(&pointer_statistic),
                                          Node::from_statistics(&character_statistic));

    let mut output = BitVec::new();
    let pointer_code = pointer_tree.encode_tree();
    let character_code = character_tree.encode_tree();
    println!("Pointer tree size {} ({} pointers)", pointer_code.len(), pointer_statistic.len());
    println!("Character tree size {} ({} characters)", character_code.len(), character_statistic.len());
    output = append_bit_vec(output, &pointer_code);
    output = append_bit_vec(output, &character_code);

    let (pointer_dictionnary, character_dictionnary) =
        (pointer_tree.to_dictionnary(BitVec::new()), character_tree.to_dictionnary(BitVec::new()));

    //println!("Pointer dictionnary : {:?}", pointer_dictionnary);
    //println!("Character dictionnary : {:?}", character_dictionnary);

    for &(pointer, character) in lz78_coded.iter() {
        output = append_bit_vec(output, &pointer_dictionnary[&pointer]);
        output = append_bit_vec(output, &character_dictionnary[&character]);
    }
    println!("Total size {}", output.len());

    serialize_bit_vec(&output)
}

pub fn decompression(content: &Vec<u8>) -> Result<Vec<u8>, &'static str> {
    if content.is_empty() {
        return Ok(content.clone());
    }
    let input = deserialize_bit_vec(content);
    let mut iter = input.iter();

    let (pointer_tree, character_tree) = (Node::decode_tree(&mut iter)?,
                                          Node::decode_tree(&mut iter)?);

    let mut lz78_coded: Vec<(u64, u8)> = Vec::new();

    if let (&Node::Leaf(pointer), &Node::Leaf(character)) = (&pointer_tree, &character_tree) {
        lz78_coded.push((pointer, character));
    } else {

        while let Some(pointer_code) = pointer_tree.scan(&mut iter)? {
            if let Some(character_code) = character_tree.scan(&mut iter)? {
                lz78_coded.push((pointer_code, character_code));
            } else {
                if let Node::Leaf(_) = pointer_tree {
                    break;
                } else {
                    return Err("No character for given pointer");
                }
            }
        }
    }

    return lz78_decoding(lz78_coded.iter());
}


#[test]
fn identity_test() {
    let input = vec![1, 2, 3, 4, 4, 4, 3, 4, 1, 2, 3, 4, 1, 2, 2, 3, 3, 4, 4, 3, 2, 1];
    println!("Input {:?}", input);
    let coded = compression(&input);
    println!("Coded {:?}", coded);
    let decoded = decompression(&coded).unwrap();
    println!("Decoded {:?}", decoded);
    assert_eq!(input, decoded)
}

#[test]
fn identity2_test() {
    let input = vec![1, 2, 3, 4, 5, 6, 7, 8];
    println!("Input {:?}", input);
    let coded = compression(&input);
    println!("Coded {:?}", coded);
    let decoded = decompression(&coded).unwrap();
    println!("Decoded {:?}", decoded);
    assert_eq!(input, decoded)
}

#[test]
fn identity3_test() {
    let input = vec![1];
    println!("Input {:?}", input);
    let coded = compression(&input);
    println!("Coded {:?}", coded);
    let decoded = decompression(&coded).unwrap();
    println!("Decoded {:?}", decoded);
    assert_eq!(input, decoded)
}

#[test]
fn identity4_test() {
    let input = vec![];
    println!("Input {:?}", input);
    let coded = compression(&input);
    println!("Coded {:?}", coded);
    let decoded = decompression(&coded).unwrap();
    println!("Decoded {:?}", decoded);
    assert_eq!(input, decoded)
}

#[test]
fn identity5_test() {
    let input = vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];
    println!("Input {:?}", input);
    let coded = compression(&input);
    println!("Coded {:?}", coded);
    let decoded = decompression(&coded).unwrap();
    println!("Decoded {:?}", decoded);
    assert_eq!(input, decoded)
}
