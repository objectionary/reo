// SPDX-FileCopyrightText: Copyright (c) 2022-2025 Yegor Bugayenko
// SPDX-License-Identifier: MIT

use crate::Universe;
use anyhow::{anyhow, Result};

/// Register all known atoms in the Universe.
pub fn register(uni: &mut Universe) {
    uni.register("org.eolang.array$length", array_length);
    uni.register("org.eolang.array$at", array_at);
}

/// EO atom `array.length`.
pub fn array_length(_uni: &mut Universe, _v: u32) -> Result<u32> {
    Err(anyhow!("Not implemented yet"))
}

/// EO atom `array.at`.
pub fn array_at(_uni: &mut Universe, _v: u32) -> Result<u32> {
    Err(anyhow!("Not implemented yet"))
}

#[test]
fn simple() {
    // assert_eq!(1, total);
}
