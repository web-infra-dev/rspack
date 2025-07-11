#![allow(clippy::unwrap_used)]

use criterion::criterion_main;
use groups::{build_chunk_graph::chunk_graph, bundle::bundle};

mod groups;

criterion_main!(bundle, chunk_graph);
