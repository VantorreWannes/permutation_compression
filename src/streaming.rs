use std::cmp::min;

use bitvec::{order::Lsb0, vec::BitVec};
use num_bigint::BigUint;
use num_traits::Zero;
use bitvec::bitvec;

use crate::binomial_cache::BINOM_CACHE;

pub struct BitRanker {
    total_ones: u64,
    total_zeros: u64,
    remaining_ones: u64,
    remaining_zeros: u64,
    current_index: BigUint,
}

impl BitRanker {
    pub fn new(total_ones: u64, total_zeros: u64) -> Self {
        BitRanker {
            total_ones,
            total_zeros,
            remaining_ones: total_ones,
            remaining_zeros: total_zeros,
            current_index: BigUint::zero(),
        }
    }

    pub fn process_chunk(&mut self, chunk: &BitVec<u64, Lsb0>) {
        for bit in chunk {
            if self.remaining_ones == 0 || self.remaining_zeros == 0 {
                break;
            }

            if *bit {
                let n = (self.remaining_ones + self.remaining_zeros - 1) as u64;
                let k = self.remaining_ones as u64;
                let c = BINOM_CACHE.get(n, k);
                self.current_index += c;
                self.remaining_ones -= 1;
            } else {
                self.remaining_zeros -= 1;
            }
        }
    }

    pub fn finalize(self) -> BigUint {
        self.current_index
    }
}

pub struct BitUnranker {
    remaining_index: BigUint,
    remaining_ones: u64,
    remaining_zeros: u64,
}

impl BitUnranker {
    pub fn new(index: BigUint, total_ones: u64, total_zeros: u64) -> Self {
        BitUnranker {
            remaining_index: index,
            remaining_ones: total_ones,
            remaining_zeros: total_zeros,
        }
    }

    pub fn next_chunk(&mut self, chunk_size: usize) -> BitVec<u64, Lsb0> {
        let chunk_size = min(chunk_size as u64, self.remaining_ones + self.remaining_zeros) as usize;
        let mut result = bitvec![u64, Lsb0; 0; chunk_size];
        
        for i in 0..chunk_size {
            if self.remaining_ones == 0 {
                result.set(i, false);
                self.remaining_zeros = self.remaining_zeros.saturating_sub(1);
                continue;
            }
            
            if self.remaining_zeros == 0 {
                result.set(i, true);
                self.remaining_ones -= 1;
                continue;
            }

            let n = (self.remaining_ones + self.remaining_zeros - 1) as u64;
            let k = self.remaining_ones as u64;
            let c = BINOM_CACHE.get(n, k);
            
            if c > self.remaining_index {
                result.set(i, false);
                self.remaining_zeros -= 1;
            } else {
                result.set(i, true);
                self.remaining_index -= &c;
                self.remaining_ones -= 1;
            }
        }
        
        result
    }
}