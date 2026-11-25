// SPDX-FileCopyrightText: Copyright (c) 2022-2026 Yegor Bugayenko
// SPDX-License-Identifier: MIT

mod array;
mod bytes;
mod data;
mod dataized;
mod int;
mod number;
#[cfg(test)]
mod tests;

use crate::Universe;

/// Register all known atoms in the Universe.
pub fn register(uni: &mut Universe) {
    int::register(uni);
    array::register(uni);
    number::register(uni);
    bytes::register(uni);
    dataized::register(uni);
}
