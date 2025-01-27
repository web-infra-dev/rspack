#![feature(trait_upcasting)]
#![allow(clippy::unwrap_used)]

use basic::basic;
use build_chunk_graph::chunk_graph;
use criterion::criterion_main;

mod basic;
mod build_chunk_graph;

criterion_main!(basic, chunk_graph);
