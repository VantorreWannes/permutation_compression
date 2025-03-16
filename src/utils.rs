use bitvec::{bitvec, vec::BitVec};
use num_bigint::BigUint;

fn binomial_coefficient(a: &BigUint, b: &BigUint) -> BigUint {
    if b > a {
        return BigUint::ZERO;
    }
    if b == &BigUint::ZERO || b == a {
        return BigUint::from(1u8);
    }
    let binding = a - b;
    let b = std::cmp::min(b, &binding); // Take advantage of symmetry
    let mut result = BigUint::from(1u8);
    let mut i = BigUint::ZERO;
    while &i < b {
        let numerator = a - &i;
        let denominator = &i + BigUint::from(1u8);
        result *= numerator;
        result /= denominator;
        i += BigUint::from(1u8);
    }
    result
}

pub fn get_permutation_index(bits: &BitVec, ones: &BigUint, zeros: &BigUint) -> BigUint {
    let mut index: BigUint = BigUint::ZERO;
    let mut ones_remaining = ones.clone();
    let mut zeros_remaining = zeros.clone();
    for bit in bits {
        if ones_remaining == BigUint::ZERO || zeros_remaining == BigUint::ZERO {
            break;
        }
        if bit == true {
            let a = &ones_remaining + &zeros_remaining - BigUint::from(1u8);
            let b = &ones_remaining;
            index += binomial_coefficient(&a, &b);
            ones_remaining -= BigUint::from(1u8);
        } else {
            zeros_remaining -= BigUint::from(1u8);
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
    let mut i = BigUint::ZERO;
    while i < total {
        if ones_remaining == BigUint::ZERO {
            result.extend_from_bitslice(&bitvec![0; zeros_remaining.try_into().unwrap()]);
            break;
        }
        if zeros_remaining == BigUint::ZERO {
            result.extend_from_bitslice(&bitvec![1; ones_remaining.try_into().unwrap()]);
            break;
        }
        let a = &ones_remaining + &zeros_remaining - BigUint::from(1u8);
        let b = &ones_remaining;
        let c = binomial_coefficient(&a, &b);
        if c > index {
            result.push(false);
            zeros_remaining -= BigUint::from(1u8);
        } else {
            result.push(true);
            index -= &c;
            ones_remaining -= BigUint::from(1u8);
        }
        i += BigUint::from(1u8);
    }
    result
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

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

    fn shannon_entropy(values: &BitVec<usize>) -> f64 {
        let mut value_counts = HashMap::new();
        for bit in values {
            *value_counts.entry(*bit).or_insert(0) += 1;
        }
        let total = values.len() as f64;
        let entropy = value_counts
            .values()
            .map(|&count| {
                let probability = count as f64 / total;
                if probability > 0.0 {
                    -probability * probability.log2()
                } else {
                    0.0
                }
            })
            .sum();
    
        entropy
    }
    #[test]
    fn test_binomial_coefficient() {
        let a = BigUint::from(50u8);
        let b = BigUint::from(25u8);
        let expected = BigUint::from(10u8);
        let result = binomial_coefficient(&a, &b);
        println!("{} {}", result, expected);
    }

    #[test]
    fn test_get_permutation_index() {
        let bits = bitvec![1, 0, 1, 1];
        let ones = BigUint::from(3u8);
        let zeros = BigUint::from(2u8);
        let expected = BigUint::from(4u8);
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
        let length: usize = 4000;
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

    #[test]
    fn test_bitvec_shannon_entropy() {
        let values = bitvec![1, 0, 1, 1];
        let result = shannon_entropy(&values);
        let expected = 0.8112781244591328;
        assert!((result - expected).abs() < 1e-10);
    }

    #[test]
    fn test_accuracy() {
        let length: usize = 8000;
        let ones = random_range(0usize..length);
        let zeros = length - ones;
        let ones = BigUint::from(ones);
        let zeros = BigUint::from(zeros);
        let expected = random_bitvec(&ones, &zeros);
        let index = get_permutation_index(&expected, &ones, &zeros);
        let original_entropy = shannon_entropy(&expected);
        let original_bit_count = expected.len();
        let compressed_bit_count = index.bits() + ones.bits();
        let expected_bit_count = original_bit_count as f64 * original_entropy;
        let bit_count_difference = compressed_bit_count  as f64 - expected_bit_count;
        let optimal_compression_accuracy = 100.0 - ((bit_count_difference / expected_bit_count) * 100.0).abs();
        println!("Original entropy: {:.2}", original_entropy);
        println!("Original bit count: {:.2}", original_bit_count);
        println!("Compressed bit count: {:.2}", compressed_bit_count);
        println!("Expected bit count: {:.2}", expected_bit_count);
        println!("Bit count difference: {:.2}", bit_count_difference);
        println!("Optimal compression accuracy: {:.2}%", optimal_compression_accuracy);
    }
}
