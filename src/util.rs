#![allow(unused)]

use std::cmp::Ordering;

use crate::def::*;

pub mod rnd {
    static mut S: usize = 88172645463325252;

    #[inline]
    pub fn next() -> usize {
        unsafe {
            S = S ^ S << 7;
            S = S ^ S >> 9;
            S
        }
    }

    #[inline]
    pub fn nextf() -> f64 {
        (next() & 4294967295) as f64 / 4294967296.
    }

    #[inline]
    pub fn gen_range(low: usize, high: usize) -> usize {
        assert!(low < high);
        (next() % (high - low)) + low
    }

    pub fn shuffle<I>(vec: &mut Vec<I>) {
        for i in 0..vec.len() {
            let j = gen_range(0, vec.len());
            vec.swap(i, j);
        }
    }
}

pub mod time {
    static mut START: f64 = -1.;
    pub fn start_clock() {
        let _ = elapsed_seconds();
    }

    #[inline]
    pub fn elapsed_seconds() -> f64 {
        let t = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        unsafe {
            if START < 0. {
                START = t;
            }
            t - START
        }
    }
}
