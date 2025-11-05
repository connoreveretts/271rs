#![allow(non_camel_case_types)]

#[derive(Debug, Clone)]
pub struct ix {
    sign: bool,
    vals: Vec<u64>,
}

fn gte_mag(a_vals: &Vec<u64>, b_vals: &Vec<u64>) -> bool {
    if a_vals.len() != b_vals.len() {
        return a_vals.len() > b_vals.len();
    }
    for i in (0..a_vals.len()).rev()  {
        if a_vals[i] != b_vals[i] {
            return a_vals[i] > b_vals[i];
        }
    }
    true
}

fn add_mag(aug_vals: &Vec<u64>, add_vals: &Vec<u64>) -> Vec<u64> {
    let max_len = aug_vals.len().max(add_vals.len());
    let mut result = Vec::with_capacity(max_len + 1);
    let mut carry: u64 = 0;
    for i in 0..max_len {
        let a = if i < aug_vals.len() { aug_vals[i] } else { 0 };
        let b = if i < add_vals.len() { add_vals[i] } else { 0 };
        let sum = a as u128 + b as u128 + carry as u128;
        result.push((sum & 0xFFFFFFFFFFFFFFFF) as u64);
        carry = (sum >> 64) as u64;
    }
    if carry > 0 {
        result.push(carry);
    }
    result
}

fn sub_mag(min_vals: &Vec<u64>, sub_vals: &Vec<u64>) -> Vec<u64> {
    let mut result = Vec::with_capacity(min_vals.len());
    let mut borrow: i128 = 0;
    for i in 0..min_vals.len() {
        let a = min_vals[i] as i128;
        let b = if i < sub_vals.len() { sub_vals[i] as i128 } else { 0 };
        let diff = a - b - borrow;
        if diff < 0 {
            result.push((diff + (1i128 << 64)) as u64);
            borrow = 1;
        } else {
            result.push(diff as u64);
            borrow = 0;
        }
    }
    while result.len() > 1 && result[result.len() - 1] == 0 {
        result.pop();
    }
    result
}

fn trim_zeros(vals: &mut Vec<u64>) {
    while vals.len() > 1 && vals[vals.len() - 1] == 0 {
        vals.pop();
    }
}

pub fn add_ix(a: &ix, b: &ix) -> ix {
    if a.sign == b.sign {
        return ix {
            sign: a.sign,
            vals: add_mag(&a.vals, &b.vals),
        };
    }
    if gte_mag(&a.vals, &b.vals) {
        let mut result_vals = sub_mag(&a.vals, &b.vals);
        trim_zeros(&mut result_vals);
        if result_vals.len() == 1 && result_vals[0] == 0 {
            return ix { sign: true, vals: result_vals };
        }
        ix { sign: a.sign, vals: result_vals }
    } else {
        let mut result_vals = sub_mag(&b.vals, &a.vals);
        trim_zeros(&mut result_vals);
        if result_vals.len() == 1 && result_vals[0] == 0 {
            return ix { sign: true, vals: result_vals };
        }
        ix { sign: b.sign, vals: result_vals }
    }
}
pub fn sub_ix(a: &ix, b: &ix) -> ix {
    let neg_b = ix { sign: !b.sign, vals: b.vals.clone() };
    add_ix(a, &neg_b)
}
pub fn mul_ix(a: &ix, b: &ix) -> ix {
    if (a.vals.len() == 1 && a.vals[0] == 0) || (b.vals.len() == 1 && b.vals[0] == 0) {
        return ix { sign: true, vals: vec![0] };
    }
    let mut result = vec![0u64; a.vals.len() + b.vals.len()];
    for i in 0..a.vals.len() {
        let mut carry: u64 = 0;
        for j in 0..b.vals.len() {
            let product = (a.vals[i] as u128) * (b.vals[j] as u128) 
                        + (result[i + j] as u128) + (carry as u128);
            result[i + j] = (product & 0xFFFFFFFFFFFFFFFF) as u64;
            carry = (product >> 64) as u64;
        }
        result[i + b.vals.len()] += carry;
    }
    while result.len() > 1 && result[result.len() - 1] == 0 {
        result.pop();
    }
    ix { sign: a.sign == b.sign, vals: result }
}
fn shl_vec(vals: &Vec<u64>, bits: usize) -> Vec<u64> {
    if bits == 0 {
        return vals.clone();
    }
    let word_shift = bits / 64;
    let bit_shift = bits % 64;
    let mut result = vec![0u64; vals.len() + word_shift + 1];
    if bit_shift == 0 {
        for i in 0..vals.len() {
            result[i + word_shift] = vals[i];
        }
    } else {
        let mut carry: u64 = 0;
        for i in 0..vals.len() {
            let shifted = vals[i] << bit_shift;
            result[i + word_shift] = shifted | carry;
            carry = vals[i] >> (64 - bit_shift);
        }
        if carry > 0 {
            result[vals.len() + word_shift] = carry;
        }
    }
    while result.len() > 1 && result[result.len() - 1] == 0 {
        result.pop();
    }
    result
}
fn bit_length(vals: &Vec<u64>) -> usize {
    if vals.is_empty() || (vals.len() == 1 && vals[0] == 0) {
        return 0;
    }
    let last_word = vals[vals.len() - 1];
    let word_bits = 64 - last_word.leading_zeros() as usize;
    (vals.len() - 1) * 64 + word_bits
}
pub fn div_ix(a: &ix, b: &ix) -> ix {
    if b.vals.len() == 1 && b.vals[0] == 0 {
        panic!("Divided by zero");
    }
    if !gte_mag(&a.vals, &b.vals) {
        return ix { sign: true, vals: vec![0] };
    }
    let a_bits = bit_length(&a.vals);
    let mut quotient = vec![0u64; (a_bits / 64) + 1];
    let mut remainder = vec![0u64];
    for i in (0..a_bits).rev() {
        remainder = shl_vec(&remainder, 1);
        let word_idx = i / 64;
        let bit_idx = i % 64;
        if (a.vals[word_idx] >> bit_idx) & 1 == 1 {
            remainder[0] |= 1;
        }
        if gte_mag(&remainder, &b.vals) {
            remainder = sub_mag(&remainder, &b.vals);
            let q_word_idx = i / 64;
            let q_bit_idx = i % 64;
            quotient[q_word_idx] |= 1u64 << q_bit_idx;
        }
    }
    while quotient.len() > 1 && quotient[quotient.len() - 1] == 0 {
        quotient.pop();
    }
    ix { sign: a.sign == b.sign, vals: quotient }
}
pub fn rem_ix(a: &ix, b: &ix) -> ix {
    if b.vals.len() == 1 && b.vals[0] == 0 {
        panic!("Divided by zero");
    }
    if !gte_mag(&a.vals, &b.vals) {
        return a.clone();
    }
    let a_bits = bit_length(&a.vals);
    let mut remainder = vec![0u64];
    for i in (0..a_bits).rev() {
        remainder = shl_vec(&remainder, 1);
        let word_idx = i / 64;
        let bit_idx = i % 64;
        if (a.vals[word_idx] >> bit_idx) & 1 == 1 {
            remainder[0] |= 1;
        }
        if gte_mag(&remainder, &b.vals) {
            remainder = sub_mag(&remainder, &b.vals);
        }
    }
    while remainder.len() > 1 && remainder[remainder.len() - 1] == 0 {
        remainder.pop();
    }
    ix { sign: a.sign, vals: remainder }
}
pub fn from_hex(s: &str) -> ix {
    let s = s.trim();
    let (sign, hex_str) = if s.starts_with("-") {
        (false, &s[1..])
    } else {
        (true, s)
    };
    let hex_str = if hex_str.starts_with("0x") || hex_str.starts_with("0X") {
        &hex_str[2..]
    } else {
        hex_str
    };
    let mut vals = Vec::new();
    let mut current: u64 = 0;
    let mut digit_count = 0;
    for ch in hex_str.chars().rev() {
        if let Some(digit) = ch.to_digit(16) {
            current |= (digit as u64) << (digit_count * 4);
            digit_count += 1;
            if digit_count == 16 {
                vals.push(current);
                current = 0;
                digit_count = 0;
            }
        }
    }
    if digit_count > 0 || vals.is_empty() {
        vals.push(current);
    }
    ix { sign, vals }
}
pub fn to_hex(n: &ix) -> String {
    if n.vals.len() == 1 && n.vals[0] == 0 {
        return String::from("0");
    }
    let mut result = String::new();
    if !n.sign {
        result.push('-');
    }
    let mut first = true;
    for word in n.vals.iter().rev() {
        if first {
            result.push_str(&format!("{:x}", word));
            first = false;
        } else {
            result.push_str(&format!("{:016x}", word));
        }
    }
    result
}
