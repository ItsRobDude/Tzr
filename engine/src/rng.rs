use rand::{Rng as RandTrait, SeedableRng};
use rand_pcg::Pcg64Mcg;

pub struct Rng {
    inner: Pcg64Mcg,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self {
            inner: Pcg64Mcg::seed_from_u64(seed),
        }
    }

    pub fn gen_range(&mut self, range: std::ops::Range<u32>) -> u32 {
        RandTrait::gen_range(&mut self.inner, range)
    }

    pub fn choose_index<T>(&mut self, slice: &[T]) -> Option<usize> {
        if slice.is_empty() {
            return None;
        }
        Some(RandTrait::gen_range(&mut self.inner, 0..slice.len()))
    }
}
