#![feature(trait_upcasting)]
#![allow(clippy::unwrap_used)]

use basic::basic;
use build_chunk_graph::chunk_graph;
use criterion::criterion_main;
use modules_10000::modules_10000;

mod basic;
mod build_chunk_graph;
mod modules_10000;

criterion_main!(basic, chunk_graph, modules_10000);
