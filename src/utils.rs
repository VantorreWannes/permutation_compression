use bitvec::{bitvec, vec::BitVec};
use num_bigint::BigUint;
use num_traits::{One, Zero};
use std::cmp::min;
use std::collections::HashMap;

lazy_static::lazy_static! {
    static ref BINOMIAL_CACHE: std::sync::Mutex<HashMap<(BigUint, BigUint), BigUint>> = std::sync::Mutex::new(HashMap::new());
}

fn binomial_coefficient(a: &BigUint, b: &BigUint) -> BigUint {
    if b > a {
        return BigUint::zero();
    }
    if b.is_zero() || b == a {
        return BigUint::one();
    }
    let binding = a - b;
    let b = min(b, &binding);

    let key = (a.clone(), b.clone());
    if let Some(cached) = BINOMIAL_CACHE.lock().unwrap().get(&key) {
        return cached.clone();
    }

    let mut result = BigUint::one();
    let mut i = BigUint::zero();
    while &i < b {
        let numerator = a - &i;
        let denominator = &i + BigUint::one();
        result *= numerator;
        result /= denominator;
        i += BigUint::one();
    }

    BINOMIAL_CACHE.lock().unwrap().insert(key, result.clone());
    result
}

pub fn get_permutation_index(bits: &BitVec, ones: &BigUint, zeros: &BigUint) -> BigUint {
    let mut index: BigUint = BigUint::zero();
    let mut ones_remaining = ones.clone();
    let mut zeros_remaining = zeros.clone();
    for bit in bits {
        if ones_remaining.is_zero() || zeros_remaining.is_zero() {
            break;
        }
        if *bit {
            let a = &ones_remaining + &zeros_remaining - BigUint::one();
            let b = &ones_remaining;
            index += binomial_coefficient(&a, &b);
            ones_remaining -= BigUint::one();
        } else {
            zeros_remaining -= BigUint::one();
        }
    }
    index
}

fn get_nth_permutation(index: &BigUint, ones: &BigUint, zeros: &BigUint) -> BitVec {
    let mut index = index.clone();
    let capacity = (ones + zeros) % BigUint::from(usize::MAX);
    let mut result = BitVec::with_capacity(capacity.try_into().unwrap());
    let mut ones_remaining = ones.clone();
    let mut zeros_remaining = zeros.clone();
    let total = ones + zeros;
    let mut i = BigUint::zero();
    while i < total {
        if ones_remaining.is_zero() {
            result.extend_from_bitslice(&bitvec![0; zeros_remaining.try_into().unwrap()]);
            break;
        }
        if zeros_remaining.is_zero() {
            result.extend_from_bitslice(&bitvec![1; ones_remaining.try_into().unwrap()]);
            break;
        }
        let a = &ones_remaining + &zeros_remaining - BigUint::one();
        let b = &ones_remaining;
        let c = binomial_coefficient(&a, &b);
        if c > index {
            result.push(false);
            zeros_remaining -= BigUint::one();
        } else {
            result.push(true);
            index -= &c;
            ones_remaining -= BigUint::one();
        }
        i += BigUint::one();
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitvec::bitvec;
    use bitvec::prelude::Lsb0;
    use rand::random_range;
    use rand::seq::SliceRandom;

    fn random_bitvec(ones: &BigUint, zeros: &BigUint) -> BitVec {
        let ones: usize = ones.try_into().unwrap();
        let zeros: usize = zeros.try_into().unwrap();
        let mut result = Vec::with_capacity(ones + zeros);
        let ones = vec![true; ones];
        let zeros = vec![false; zeros];
        result.extend_from_slice(&ones);
        result.extend_from_slice(&zeros);
        let mut rng = rand::rng();
        result.shuffle(&mut rng);
        BitVec::from_iter(result)
    }

    #[test]
    fn test_binomial_coefficient() {
        let a = BigUint::from(50u8);
        let b = BigUint::from(25u8);
        let expected = BigUint::from(126410606437752u64);
        let result = binomial_coefficient(&a, &b);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_permutation_index() {
        let bits = bitvec![1, 0, 1, 1, 0];
        let ones = BigUint::from(3u8);
        let zeros = BigUint::from(2u8);
        let expected = BigUint::from(6u8);
        let result = get_permutation_index(&bits, &ones, &zeros);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_nth_permutation() {
        let expected = bitvec![1, 0, 1, 1, 1, 1, 0, 1, 1, 1];
        let ones = BigUint::from(expected.count_ones());
        let zeros = BigUint::from(expected.count_zeros());
        let index = get_permutation_index(&expected, &ones, &zeros);
        let result = get_nth_permutation(&index, &ones, &zeros);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_cost() {
        let length: usize = 10000;
        let ones = random_range(0usize..length);
        let zeros = length - ones;
        let ones = BigUint::from(ones);
        let zeros = BigUint::from(zeros);
        let expected = random_bitvec(&ones, &zeros);
        let index = get_permutation_index(&expected, &ones, &zeros);
        let index_bits = index.bits();
        let ones_bits = ones.bits();
        println!(
            "Cost: {} + {} == {}",
            index_bits,
            ones_bits,
            index_bits + ones_bits
        );
    }
    
}
