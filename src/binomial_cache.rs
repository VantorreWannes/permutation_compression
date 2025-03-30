use lru::LruCache;
use num_bigint::BigUint;
use num_traits::One;
use std::cmp::min;
use std::num::NonZero;
use std::sync::Mutex;

pub struct BinomialCache {
    cache: Mutex<LruCache<(u64, u64), BigUint>>,
    precomputed: Vec<Vec<Option<BigUint>>>,
}

impl BinomialCache {
    const PRECOMPUTE_LIMIT: usize = 256;

    pub fn new() -> Self {
        let mut precomputed = vec![vec![None; Self::PRECOMPUTE_LIMIT]; Self::PRECOMPUTE_LIMIT];
        for n in 0..Self::PRECOMPUTE_LIMIT {
            for k in 0..=n {
                precomputed[n][k] = Some(compute_binomial(n as u64, k as u64));
            }
        }
        
        BinomialCache {
            cache: Mutex::new(LruCache::new(NonZero::new(1024).unwrap())),
            precomputed,
        }
    }

    pub fn get(&self, n: u64, k: u64) -> BigUint {
        let k = min(k, n - k);
        
        if n < Self::PRECOMPUTE_LIMIT as u64 && k < Self::PRECOMPUTE_LIMIT as u64 {
            return self.precomputed[n as usize][k as usize].clone().unwrap();
        }

        let mut cache = self.cache.lock().unwrap();
        if let Some(val) = cache.get(&(n, k)) {
            return val.clone();
        }

        let result = compute_binomial(n, k);
        cache.put((n, k), result.clone());
        result
    }
}

fn compute_binomial(n: u64, k: u64) -> BigUint {
    if k == 0 || k == n {
        return BigUint::one();
    }
    
    let mut result = BigUint::one();
    let k = min(k, n - k);
    
    for i in 1..=k {
        result = result * (n - k + i) / i;
    }
    
    result
}

lazy_static::lazy_static! {
    pub static ref BINOM_CACHE: BinomialCache = BinomialCache::new();
}