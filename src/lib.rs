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
use std::collections::btree_map::Entry;

pub fn compression_huffman(content: &[u8]) -> Vec<u8> {
    if content.is_empty() {
        return vec![];
    }
    let mut stat = BTreeMap::new();

    for &byte in content.iter() {
        *stat.entry(byte).or_insert(0) += 1;
    }
    if stat.len() == 1 {
        for byte in 0..256 {
            if let Entry::Vacant(entry) = stat.entry(byte as u8) {
                entry.insert(1);
                break;
            }
        }
    }

    let tree = Node::from_statistics(&stat);

    let mut output = BitVec::new();
    output = append_bit_vec(output, &tree.encode_tree());

    let dico = tree.to_dictionnary(BitVec::new());

    for &byte in content.iter() {
        output = append_bit_vec(output, &dico[&byte]);
    }

    serialize_bit_vec(&output)
}

pub fn decompression_huffman(content: &[u8]) -> Result<Vec<u8>, &'static str> {
    if content.is_empty() {
        return Ok(vec![]);
    }
    let input = deserialize_bit_vec(content);
    let mut iter = input.iter();

    let tree: Node<u8> = Node::decode_tree(&mut iter)?;

    let mut output: Vec<u8> = Vec::new();

    'b: loop {
        let byte = match tree.scan(&mut iter) {
            Ok(Some(byte)) => byte,
            Ok(None) => {
                break 'b;
            }
            Err(err) => return Err(err),
        };
        output.push(byte);
    }
    Ok(output)
}

pub fn compression_lz77<I>(iter: I) -> Vec<u8>
    where I: Iterator<Item = u8>
{
    let lz77_coded: Vec<(u16, u8)> = lz77_coding(iter)
        .map(|(ptr, len, byte)| {
            assert!(ptr < 4096, "ptr = {}, len = {}", ptr, len);
            assert!(len < 16, "ptr = {}, len = {}", ptr, len);
            (ptr << 4 | len as u16, byte)
        })
        .collect();

    let mut output = Vec::new();

    for &(ptr_len, byte) in &lz77_coded {
        let x1 = (ptr_len & 0xff) as u8;
        let x2 = ((ptr_len >> 8) & 0xff) as u8;
        output.push(x1);
        output.push(x2);
        output.push(byte);
    }

    output
}

pub fn decompression_lz77(content: &[u8]) -> Result<Vec<u8>, &'static str> {
    let mut iter = content.iter();

    let mut lz77_coded: Vec<(u16, u8, u8)> = Vec::new();

    while let Some(&x1) = iter.next() {
        // loop {
        //     let x1 = match iter.next() {
        //         Some(&x1) => x1,
        //         None => break
        //     };
        let x2 = *iter.next().ok_or("Unexpected EOF")?;
        let byte = *iter.next().ok_or("Unexpected EOF")?;
        let ptr_len = ((x2 as u16) << 8) | x1 as u16;
        let triplet = (ptr_len >> 4, (ptr_len & 0b1111) as u8, byte);
        lz77_coded.push(triplet);
    }

    Ok(lz77_decoding(lz77_coded.iter()).collect())
}

pub fn compression_lz78(content: &[u8]) -> Vec<u8> {
    if content.is_empty() {
        return Vec::new();
    }
    let lz78_coded: Vec<(u64, u8)> = lz78_coding(content.iter());

    let (mut pointer_statistic, mut character_statistic): (BTreeMap<u64, u64>,
                                                           BTreeMap<u8, u64>) = (BTreeMap::new(),
                                                                                 BTreeMap::new());

    for &(pointer, character) in &lz78_coded {
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

    for &(pointer, character) in &lz78_coded {
        output = append_bit_vec(output, &pointer_dictionnary[&pointer]);
        output = append_bit_vec(output, &character_dictionnary[&character]);
    }
    println!("Total size {}", output.len());

    serialize_bit_vec(&output)
}

pub fn decompression_lz78(content: &[u8]) -> Result<Vec<u8>, &'static str> {
    if content.is_empty() {
        return Ok(Vec::new());
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
            } else if let Node::Leaf(_) = pointer_tree {
                break;
            } else {
                return Err("No character for given pointer");
            }
        }
    }

    lz78_decoding(lz78_coded.iter())
}


#[test]
fn identity_test() {
    let input = vec![1, 2, 3, 4, 4, 4, 3, 4, 1, 2, 3, 4, 1, 2, 2, 3, 3, 4, 4, 3, 2, 1];
    println!("Input {:?}", input);
    let coded = compression_huffman(&input);
    println!("Coded {:?}", coded);
    let decoded = decompression_huffman(&coded).unwrap();
    println!("Decoded {:?}", decoded);
    assert_eq!(input, decoded)
}

#[test]
fn identity2_test() {
    let input = vec![1, 2, 3, 4, 5, 6, 7, 8];
    println!("Input {:?}", input);
    let coded = compression_huffman(&input);
    println!("Coded {:?}", coded);
    let decoded = decompression_huffman(&coded).unwrap();
    println!("Decoded {:?}", decoded);
    assert_eq!(input, decoded)
}

#[test]
fn identity3_test() {
    let input = vec![1];
    println!("Input {:?}", input);
    let coded = compression_huffman(&input);
    println!("Coded {:?}", coded);
    let decoded = decompression_huffman(&coded).unwrap();
    println!("Decoded {:?}", decoded);
    assert_eq!(input, decoded)
}

#[test]
fn identity4_test() {
    let input = vec![];
    println!("Input {:?}", input);
    let coded = compression_huffman(&input);
    println!("Coded {:?}", coded);
    let decoded = decompression_huffman(&coded).unwrap();
    println!("Decoded {:?}", decoded);
    assert_eq!(input, decoded)
}

#[test]
fn identity5_test() {
    let input = vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];
    println!("Input {:?}", input);
    let coded = compression_huffman(&input);
    println!("Coded {:?}", coded);
    let decoded = decompression_huffman(&coded).unwrap();
    println!("Decoded {:?}", decoded);
    assert_eq!(input, decoded)
}
