// SPDX-FileCopyrightText: Copyright (c) 2022-2025 Yegor Bugayenko
// SPDX-License-Identifier: MIT

#![deny(warnings)]

pub mod compiler;
pub mod runtime;

use log::LevelFilter;
use simple_logger::SimpleLogger;

#[ctor::ctor]
fn init() {
    SimpleLogger::new()
        .without_timestamps()
        .with_level(LevelFilter::Trace)
        .init()
        .unwrap();
}
