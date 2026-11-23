// SPDX-FileCopyrightText: Copyright (c) 2022-2026 Yegor Bugayenko
// SPDX-License-Identifier: MIT

use crate::Universe;
use anyhow::{anyhow, Result};

/// Register all known atoms in the Universe.
pub fn register(uni: &mut Universe) {
    uni.register("L_array_length", array_length);
    uni.register("L_array_at", array_at);
}

/// EO atom `array.length`.
pub fn array_length(_uni: &mut Universe, _c: usize) -> Result<usize> {
    Err(anyhow!("Not implemented yet"))
}

/// EO atom `array.at`.
pub fn array_at(_uni: &mut Universe, _c: usize) -> Result<usize> {
    Err(anyhow!("Not implemented yet"))
}

#[test]
fn simple() {
    // assert_eq!(1, total);
}
