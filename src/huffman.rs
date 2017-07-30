use bit_vec::{BitVec, Iter};
use bitvec_util::*;
use binary_heap_compare::BinaryHeapCompare;
use std::collections::BTreeMap;
use std;

#[derive(Debug)]
pub enum Node<T> {
    Leaf(T),
    Branch(Box<Node<T>>, Box<Node<T>>),
}

impl<T: Ord + Copy> Node<T> {
    #[allow(dead_code)]
    pub fn from_statistics(statistics: &BTreeMap<T, u64>) -> Node<T> {
        let mut heap: BinaryHeapCompare<(u64, Node<T>), _> =
            BinaryHeapCompare::new(|x: &(u64, _), y: &(u64, _)| y.0.cmp(&(x.0)));

        for (&key, &value) in statistics.iter() {
            if value != 0 {
                heap.push((value, Node::Leaf(key)));
            }
        }

        while heap.len() > 1 {
            let a = heap.pop().unwrap();
            let b = heap.pop().unwrap();
            heap.push((a.0 + b.0, Node::Branch(Box::new(a.1), Box::new(b.1))));
        }

        heap.pop().unwrap().1
    }

    #[allow(dead_code)]
    pub fn to_dictionnary(self, current_word: BitVec) -> BTreeMap<T, BitVec> {
        match self {
            Node::Leaf(character) => {
                let mut dictionnary = BTreeMap::new();
                dictionnary.insert(character, current_word);
                dictionnary
            }
            Node::Branch(left_path, right_path) => {
                let mut left_path_word = current_word.clone();
                left_path_word.push(false);
                let mut left_dictionnary = left_path.to_dictionnary(left_path_word);

                let mut right_path_word = current_word.clone();
                right_path_word.push(true);
                let mut right_dictionnary = right_path.to_dictionnary(right_path_word);

                left_dictionnary.append(&mut right_dictionnary);
                left_dictionnary
            }
        }
    }

    #[allow(dead_code)]
    pub fn scan(&self, iter: &mut Iter) -> Result<Option<T>, &'static str> {
        match self {
            &Node::Leaf(character) => Ok(Some(character)),
            &Node::Branch(ref node_0, ref node_1) => {
                if let Some(bit) = iter.next() {
                    let result = if bit { node_1 } else { node_0 }.scan(iter)?;
                    match result {
                        Some(character) => Ok(Some(character)),
                        None => Err("unexpected EOF"),
                    }
                } else {
                    Ok(None)
                }
            }
        }
    }
}

pub trait IntoBitVec {
    fn encode(&self) -> BitVec;
}

impl IntoBitVec for u8 {
    fn encode(&self) -> BitVec {
        BitVec::from_bytes(&[*self])
    }
}

impl IntoBitVec for u64 {
    fn encode(&self) -> BitVec {
        let mut x = *self;
        let mut xs = [0; 8];
        for i in 0..8 {
            xs[7 - i] = (x & 0xff) as u8;
            x >>= 8;
        }
        BitVec::from_bytes(&xs)
    }
}

pub trait FromBitVec
    where Self: std::marker::Sized
{
    fn decode(&mut Iter) -> Result<Self, &'static str>;
}

impl FromBitVec for u8 {
    fn decode(iter: &mut Iter) -> Result<u8, &'static str> {
        let mut x = 0u8;
        for _ in 0..8 {
            match iter.next() {
                Some(true) => { x <<= 1; x |= 1; }
                Some(false) => { x <<= 1; }
                None => { return Err("unexpected EOF while decoding u8"); }
            }
        }
        Ok(x)
    }
}

impl FromBitVec for u64 {
    fn decode(iter: &mut Iter) -> Result<u64, &'static str> {
        let mut x = 0u64;
        for _ in 0..64 {
            match iter.next() {
                Some(true) => { x <<= 1; x |= 1; }
                Some(false) => { x <<= 1; }
                None => { return Err("unexpected EOF while decoding u64"); }
            }
        }
        Ok(x)
    }
}

impl<T: IntoBitVec + FromBitVec + Copy> Node<T> {
    #[allow(dead_code)]
    pub fn encode_tree(&self) -> BitVec {
        match self {
            &Node::Leaf(ref character) => {
                let mut output = BitVec::new();
                output.push(true);
                append_bit_vec(output, &character.encode())
            }
            &Node::Branch(ref left_node, ref right_node) => {
                let mut output = BitVec::new();
                output.push(false);
                append_bit_vec(append_bit_vec(output, &left_node.encode_tree()),
                               &right_node.encode_tree())
            }
        }
    }

    #[allow(dead_code)]
    pub fn decode_tree(iter: &mut Iter) -> Result<Node<T>, &'static str> {
        if let Some(val) = iter.next() {
            if val {
                Ok(Node::Leaf(T::decode(iter)?))
            } else {
                Ok(Node::Branch(Box::new(Self::decode_tree(iter)?),
                                Box::new(Self::decode_tree(iter)?)))
            }
        } else {
            Err("unexpected EOF")
        }
    }
}
