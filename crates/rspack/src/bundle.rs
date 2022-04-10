use std::{collections::HashMap, sync::Arc};

use dashmap::DashSet;

use crate::{bundler::BundleOptions, chunk::Chunk, graph, structs::OutputChunk};

#[non_exhaustive]
pub struct Bundle {
    pub graph: graph::Graph,
    pub output_options: Arc<BundleOptions>,
}

impl Bundle {
    pub fn new(graph: graph::Graph, output_options: Arc<BundleOptions>) -> Self {
        Self {
            graph,
            output_options,
        }
    }

    fn generate_chunks(&self) -> Vec<Chunk> {
        let entries = DashSet::new();
        self.graph.entry_indexs.iter().for_each(|entry| {
            let entry = self.graph.module_graph[*entry].to_owned();
            entries.insert(entry);
        });

        let chunks = vec![Chunk {
            id: Default::default(),
            order_modules: self
                .graph
                .ordered_modules
                .clone()
                .into_iter()
                .map(|idx| self.graph.module_graph[idx].clone())
                .collect(),
            symbol_box: self.graph.mark_box.clone(),
            entries,
        }];

        chunks
    }

    pub fn generate(&mut self) -> HashMap<String, OutputChunk> {
        let mut chunks = self.generate_chunks();

        chunks.iter_mut().for_each(|chunk| {
            chunk.id = chunk.generate_id(&self.output_options);
        });

        chunks
            .iter_mut()
            .map(|chunk| {
                let chunk = chunk.render(&self.output_options, &mut self.graph.module_by_id);
                (
                    chunk.file_name.clone(),
                    OutputChunk {
                        code: chunk.code,
                        file_name: chunk.file_name,
                    },
                )
            })
            .collect()
    }
}
