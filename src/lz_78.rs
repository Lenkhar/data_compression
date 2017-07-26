use std::collections::BTreeMap;

#[allow(dead_code)]
pub fn lz78_coding<'a, I>(iter: I) -> Vec<(u64, u8)>
    where I: Iterator<Item = &'a u8>
{
    let mut dictionnary: BTreeMap<Vec<u8>, u64> = BTreeMap::new();
    let mut output: Vec<(u64, u8)> = Vec::new();

    let mut word: Vec<u8> = Vec::new();
    let mut last_pointer: u64 = 0;
    for &item in iter {
        word.push(item);
        if let Some(&pointer) = dictionnary.get(&word) {
            last_pointer = pointer;
        } else {
            let index: u64 = (dictionnary.len() + 1) as u64;
            dictionnary.insert(word, index);
            output.push((last_pointer, item));
            last_pointer = 0;
            word = Vec::new();
        }

    }
    if let Some(last_char) = word.pop() {
        let pointer = if let Some(&pointer) = dictionnary.get(&word) {
            pointer
        } else {
            0
        };
        output.push((pointer, last_char));
    }
    return output;
}

#[allow(dead_code)]
pub fn lz78_decoding<'a, I>(iter: I) -> Result<Vec<u8>, &'static str>
    where I: Iterator<Item = &'a (u64, u8)>
{
    let mut dictionnary: BTreeMap<u64, Vec<u8>> = BTreeMap::new();
    dictionnary.insert(0, Vec::new());
    let mut output: Vec<u8> = Vec::new();

    for &(pointer, character) in iter {
        let mut word = match dictionnary.get(&pointer) {
            Some(tmp_word) => tmp_word.clone(),
            None => return Err("Invalid entry"),
        };

        word.push(character);
        let index = dictionnary.len();
        dictionnary.insert(index as u64, word.clone());
        output.append(&mut word);


    }
    return Ok(output);
}

#[test]
fn lz_78_testing() {
    let input: Vec<u8> = vec![1, 1, 2, 1, 2, 2, 2, 1, 2, 1, 1, 2, 1, 2, 2, 2, 1, 2, 2, 1, 2, 2, 6,
                              5, 4, 3, 6, 5, 4, 3, 2, 3, 4, 4, 4, 4, 4, 5, 6, 7, 8, 4, 9, 9, 9, 9,
                              9, 9, 9, 9, 9, 9, 9, 9, 7];
    println!("Input {:?}", input);
    let coded = lz78_coding(input.iter());
    println!("Coded {:?}", coded);
    let decoded = lz78_decoding(coded.iter());

    println!("Decoded {:?}", decoded);
    assert_eq!(input, decoded.unwrap());
}
