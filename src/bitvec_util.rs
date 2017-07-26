use bit_vec::BitVec;

pub fn append_bit_vec(mut a: BitVec, b: &BitVec) -> BitVec {
    a.extend(b.iter());
    a
}

pub fn serialize_bit_vec(x: &BitVec) -> Vec<u8> {
    let modulo = (x.len() % 8) as u8;
    let mut output = x.to_bytes();
    output.push(if modulo == 0 { 0 } else { 8 - modulo });
    return output;
}

pub fn deserialize_bit_vec(x: &[u8]) -> BitVec {
    let mut bits = BitVec::from_bytes(&x[..x.len() - 1]);
    let len = bits.len() - x[x.len() - 1] as usize;
    bits.truncate(len);
    bits
}

#[allow(dead_code)]
pub fn pop_front_bit_vec(mut to_pop: BitVec) -> BitVec {
    for i in 0..to_pop.len() - 1 {
        let next = to_pop[i + 1];
        to_pop.set(i, next);
    }
    to_pop.pop();
    return to_pop;
}
