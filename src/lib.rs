extern crate binary_heap_compare;
extern crate bit_vec;
use bit_vec::BitVec;
mod huffman;
use huffman::*;
mod bitvec_util;
mod lz_77;
mod lz_78;
use lz_77::{lz77_coding, lz77_decoding};
use lz_78::{lz78_coding, lz78_decoding};
use bitvec_util::*;
use std::collections::BTreeMap;


pub fn compression_lz77<I>(iter: I) -> Vec<u8>
    where I: Iterator<Item = u8>
{
    let lz77_coded: Vec<(u16, u8)> = lz77_coding(iter)
        .map(|(ptr, len, byte)| {
            assert!(ptr < 4096);
            assert!(len < 32);
            (ptr << 4 | len as u16, byte)
        })
        .collect();

    let mut stat = BTreeMap::new();

    for &(ptr_len, byte) in lz77_coded.iter() {
        *stat.entry(byte).or_insert(0) += 1;
        *stat.entry((ptr_len & 0xff) as u8).or_insert(0) += 1;
        *stat.entry(((ptr_len >> 8) & 0xff) as u8).or_insert(0) += 1;
    }

    let tree = Node::from_statistics(&stat);

    let mut output = BitVec::new();
    output = append_bit_vec(output, &tree.encode_tree());

    let dico = tree.to_dictionnary(BitVec::new());

    for &(ptr_len, byte) in lz77_coded.iter() {
        let x1 = (ptr_len & 0xff) as u8;
        let x2 = ((ptr_len >> 8) & 0xff) as u8;
        output = append_bit_vec(output, &dico[&x1]);
        output = append_bit_vec(output, &dico[&x2]);
        output = append_bit_vec(output, &dico[&byte]);
    }

    serialize_bit_vec(&output)
}

pub fn decompression_lz77(content: &Vec<u8>) -> Result<Vec<u8>, &'static str> {
    if content.is_empty() {
        return Ok(content.clone());
    }
    let input = deserialize_bit_vec(content);
    let mut iter = input.iter();

    let tree: Node<u8> = Node::decode_tree(&mut iter)?;

    let mut lz77_coded: Vec<(u16, u8, u8)> = Vec::new();

    loop {
        let x1 = match tree.scan(&mut iter) {
            Ok(Some(x1)) => x1,
            Ok(None) => {
                break;
            }
            Err(err) => return Err(err),
        };
        let x2 = match tree.scan(&mut iter) {
            Ok(Some(x2)) => x2,
            Ok(None) => return Err("Problem"),
            Err(err) => return Err(err),
        };
        let byte = match tree.scan(&mut iter) {
            Ok(Some(byte)) => byte,
            Ok(None) => return Err("Problem"),
            Err(err) => return Err(err),
        };
        let ptr_len = ((x2 as u16) << 8) | x1 as u16;
        let triplet = (ptr_len >> 4, (ptr_len & 0b1111) as u8, byte);
        lz77_coded.push(triplet);
    }

    Ok(lz77_decoding(lz77_coded.iter()).collect())
}

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
    println!("Pointer tree size {} ({} pointers)",
             pointer_code.len(),
             pointer_statistic.len());
    println!("Character tree size {} ({} characters)",
             character_code.len(),
             character_statistic.len());
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
