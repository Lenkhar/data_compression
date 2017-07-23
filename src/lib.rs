#![allow(dead_code)]
extern crate binary_heap_compare;
extern crate bit_vec;
use binary_heap_compare::BinaryHeapCompare;
use bit_vec::BitVec;


enum Node {
    Leave(u8),
    Branch(Box<Node>, Box<Node>),
}


fn scan_tree(current_node: Node, current_word: BitVec, dictionnary: &mut Vec<BitVec>) {
    match current_node {
        Node::Leave(character) => dictionnary[character as usize] = current_word,
        Node::Branch(left_path, right_path) => {
            let mut left_path_word = current_word.clone();
            left_path_word.push(false);
            scan_tree(*left_path, left_path_word, dictionnary);

            let mut right_path_word = current_word.clone();
            right_path_word.push(true);
            scan_tree(*right_path, right_path_word, dictionnary);
        }
    }
}

fn append_bit_vec(is_appended: &mut BitVec, to_append: &BitVec) {
    for i in 0..to_append.len() {
        is_appended.push(to_append[i]);
    }
}

fn serialize_bit_vec(x: &BitVec) -> Vec<u8> {
    let len = x.len() as u64;
    let raw_bytes: [u8; 8] = unsafe { std::mem::transmute(len) };
    let mut output = raw_bytes.to_vec();
    output.append(&mut x.to_bytes());
    return output;
}

fn unserialize_bit_vec(x: &[u8]) -> (BitVec, usize) {
    let mut raw_bytes: [u8; 8] = [0; 8]; // [x[0], x[1], x[2], x[3], x[4], x[5], x[6], x[7]];
    raw_bytes.copy_from_slice(&x[..8]);
    let len: u64 = unsafe { std::mem::transmute(raw_bytes) };
    let bit_vec_len = ((len + 7) / 8) as usize;
    let mut bits = BitVec::from_bytes(&x[8..8+bit_vec_len]);
    bits.truncate(len as usize);
    (bits, 8 + bit_vec_len)
}

pub fn compression(content: Vec<u8>) -> Vec<u8> {
    let mut statistique = vec![0;256];
    for &item in content.iter() {
        statistique[item as usize] += 1;
    }

    let mut heap: BinaryHeapCompare<(u64, Node), _> =
        BinaryHeapCompare::new(|x: &(u64, Node), y: &(u64, Node)| y.0.cmp(&(x.0)));

    for index in 0..statistique.len() {
        if statistique[index] != 0 {
            heap.push((statistique[index], Node::Leave(index as u8)));
        }
    }

    while heap.len() > 1 {
        let a = heap.pop().unwrap();
        let b = heap.pop().unwrap();
        heap.push((a.0 + b.0, Node::Branch(Box::new(a.1), Box::new(b.1))));
    }

    let mut dictionnary = vec![BitVec::new();256];
    let root_word = BitVec::new();

    if let Some(final_tree) = heap.pop() {
        scan_tree(final_tree.1, root_word, &mut dictionnary);
    }


    let mut output = Vec::new();
    for item in dictionnary.iter() {
        output.append(&mut serialize_bit_vec(item));
    }

    let mut translation = BitVec::new();
    for &item in content.iter() {
        append_bit_vec(&mut translation, &dictionnary[item as usize]);
    }
    output.append(&mut serialize_bit_vec(&translation));
    return output;
}

fn pop_front_bit_vec(mut to_pop: BitVec) -> BitVec {
    for i in 0..to_pop.len() - 1 {
        let next = to_pop[i + 1];
        to_pop.set(i, next);
    }
    to_pop.pop();
    return to_pop;
}

fn build_tree(tree_seed: Vec<(u8, BitVec)>) -> Node {
    if tree_seed.len() == 1 {
        return Node::Leave(tree_seed[0].0);
    } else {
        let mut left = Vec::new();
        let mut right = Vec::new();

        for item in tree_seed {
            [&mut left, &mut right][item.1[0] as usize].push((item.0, pop_front_bit_vec(item.1)));
        }

        return Node::Branch(Box::new(build_tree(left)), Box::new(build_tree(right)));
    }


}

pub fn decompression(content: Vec<u8>) -> Vec<u8> {
    let mut dictionnary: Vec<(u8, BitVec)> = Vec::new();
    let mut start = 0;
    for i in 0..256 {
        let (character, new_start) = unserialize_bit_vec(&content[start..]);
        start += new_start;
        if character.len() > 0 {
            dictionnary.push((i as u8, character));
        }
    }

    let tree = build_tree(dictionnary);
    let (translation, _) = unserialize_bit_vec(&content[start..]);

    let mut output: Vec<u8> = Vec::new();
    let mut reading_head = &tree;
    for bit in translation.iter() {
        if let &Node::Branch(ref node_0, ref node_1) = reading_head {
            if bit {
                reading_head = &node_1;
            } else {
                reading_head = &node_0;
            }
            if let &Node::Leave(character) = reading_head {
                output.push(character);
                reading_head = &tree;
            }
        }
    }
    return output;
}
