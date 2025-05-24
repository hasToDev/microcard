// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use rand::{rngs::StdRng, Rng, SeedableRng};
use std::sync::{Mutex, OnceLock};
static RNG: OnceLock<Mutex<StdRng>> = OnceLock::new();

fn truncate(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        None => s,
        Some((idx, _)) => &s[..idx],
    }
}

fn get_seed_array(hash: String, timestamp: String) -> [u8; 32] {
    // produce a seed array using hash and system time
    let concatenated_timestamp = format!("{}{}{}{}{}", hash, hash, timestamp, timestamp, timestamp);
    let timestamp_str = truncate(concatenated_timestamp.as_str(), 32);
    <[u8; 32]>::try_from(timestamp_str.as_bytes()).unwrap()
}

pub fn get_custom_rng(hash: String, timestamp: String) -> Result<StdRng, getrandom::Error> {
    Ok(RNG
        .get_or_init(|| Mutex::new(StdRng::from_seed(get_seed_array(hash, timestamp))))
        .lock()
        .expect("failed to get RNG lock")
        .clone())
}

pub fn get_random_value(min: u8, max: u8, hash: String, timestamp: String) -> Result<u8, getrandom::Error> {
    Ok(RNG
        .get_or_init(|| Mutex::new(StdRng::from_seed(get_seed_array(hash, timestamp))))
        .lock()
        .expect("failed to get RNG lock")
        .gen_range(min..max))
}
