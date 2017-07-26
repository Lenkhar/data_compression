use bit_vec::{BitVec, Iter};
use bitvec_util::*;
use binary_heap_compare::BinaryHeapCompare;

#[derive(Debug)]
pub enum Node {
    Leaf(u8),
    Branch(Box<Node>, Box<Node>),
}

impl Node {
    pub fn encode_tree(&self) -> BitVec {
        match *self {
            Node::Leaf(character) => {
                let mut output = BitVec::new();
                output.push(true);
                append_bit_vec(output, &BitVec::from_bytes(&[character]))
            }
            Node::Branch(ref left_node, ref right_node) => {
                let mut output = BitVec::new();
                output.push(false);
                append_bit_vec(append_bit_vec(output, &left_node.encode_tree()),
                               &right_node.encode_tree())
            }
        }
    }

    pub fn decode_tree(iter: &mut Iter) -> Node {
        match iter.next() {
            Some(val) => {
                if val {
                    let mut character: u8 = 0;
                    for _ in 0..8 {
                        character <<= 1;
                        character |= iter.next().unwrap() as u8;
                    }
                    Node::Leaf(character)
                } else {
                    Node::Branch(Box::new(Self::decode_tree(iter)),
                                 Box::new(Self::decode_tree(iter)))
                }
            }
            None => panic!(),
        }
    }

    pub fn from_statistics_u8(statistique: &[u64; 256]) -> Node {
        let mut heap: BinaryHeapCompare<(u64, Node), _> =
            BinaryHeapCompare::new(|x: &(u64, Node), y: &(u64, Node)| y.0.cmp(&(x.0)));

        for index in 0..statistique.len() {
            if statistique[index] != 0 {
                heap.push((statistique[index], Node::Leaf(index as u8)));
            }
        }

        while heap.len() > 1 {
            let a = heap.pop().unwrap();
            let b = heap.pop().unwrap();
            heap.push((a.0 + b.0, Node::Branch(Box::new(a.1), Box::new(b.1))));
        }

        heap.pop().unwrap().1
    }
}

pub fn scan_tree(current_node: Node, current_word: BitVec, dictionnary: &mut Vec<BitVec>) {
    match current_node {
        Node::Leaf(character) => dictionnary[character as usize] = current_word,
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
