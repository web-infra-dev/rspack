#![feature(trait_upcasting)]
#![allow(clippy::unwrap_used)]

use basic::basic;
use build_chunk_graph::chunk_graph;
use criterion::criterion_main;
use flag_dependency_exports_plugin::flag_dependency_exports_plugin;

mod basic;
mod build_chunk_graph;
mod flag_dependency_exports_plugin;

criterion_main!(basic, chunk_graph, flag_dependency_exports_plugin);
