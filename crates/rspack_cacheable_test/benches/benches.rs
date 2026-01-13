#![allow(clippy::unwrap_used)]

mod portable_path;
mod portable_string;

use criterion::criterion_main;

criterion_main!(portable_path::benches, portable_string::benches);
