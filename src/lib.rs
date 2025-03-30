use num_bigint::BigUint;
use streaming::{BitRanker, BitUnranker};

pub mod binomial_cache;
pub mod streaming;
use bitvec::{bitvec, order::Lsb0};

fn stream_rank(bits: impl Iterator<Item = bool>, ones: u64, zeros: u64) -> BigUint {
    let mut ranker = BitRanker::new(ones, zeros);
    let mut chunk = bitvec![u64, Lsb0;];

    for bit in bits {
        chunk.push(bit);
        if chunk.len() == 64 {
            ranker.process_chunk(&chunk);
            chunk.clear();
        }
    }

    if !chunk.is_empty() {
        ranker.process_chunk(&chunk);
    }

    ranker.finalize()
}

fn stream_unrank(index: BigUint, ones: u64, zeros: u64) -> impl Iterator<Item = bool> {
    let mut unranker = BitUnranker::new(index, ones, zeros);
    std::iter::from_fn(move || {
        let chunk = unranker.next_chunk(64);
        if chunk.is_empty() {
            None
        } else {
            Some(chunk.into_iter())
        }
    })
    .flatten()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitvec::{
        vec::BitVec,
        view::BitViewSized,
    };
    use rand::{random_range, seq::SliceRandom};

    fn random_bitvec(ones: usize, zeros: usize) -> BitVec {
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
    fn test_stream_rank() {
        let bits = bitvec![1, 0, 1, 1, 0];
        let ones = 3;
        let zeros = 2;
        let result = stream_rank(bits.iter().map(|bit| *bit), ones, zeros);
        let expected = BigUint::from(6u8);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_stream_unrank() {
        let length: usize = 1000;
        let ones = random_range(0usize..length);
        let zeros = length - ones;
        let expected = random_bitvec(ones, zeros);
        let index = stream_rank(expected.iter().map(|bit| *bit), ones as u64, zeros as u64);
        let result = stream_unrank(index, ones as u64, zeros as u64).collect::<BitVec>();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_cost() {
        let length: usize = 1000;
        let ones = random_range(0usize..length);
        let zeros = length - ones;
        let expected = random_bitvec(ones, zeros);
        let index = stream_rank(expected.iter().map(|bit| *bit), ones as u64, zeros as u64);
        let index_bits = index.bits();
        let ones_bits = ones.into_bitarray::<Lsb0>().count_ones();
        println!(
            "Cost: {} + {} == {}",
            index_bits,
            ones_bits,
            index_bits + ones_bits as u64
        );
    }
}
