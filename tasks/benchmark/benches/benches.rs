#![allow(clippy::unwrap_used)]

use criterion::criterion_main;
use groups::{basic_build::basic, build_chunk_graph::chunk_graph, modules_1000::modules_1000};

mod groups;

criterion_main!(basic, chunk_graph, modules_1000);
