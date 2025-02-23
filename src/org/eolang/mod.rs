// SPDX-FileCopyrightText: Copyright (c) 2022-2025 Yegor Bugayenko
// SPDX-License-Identifier: MIT

mod array;
mod int;

use crate::Universe;

/// Register all known atoms in the Universe.
pub fn register(uni: &mut Universe) {
    int::register(uni);
    array::register(uni);
}
