use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use dashmap::DashSet;

use crate::{
    bundler::BundleOptions, chunk::Chunk, mark_box::MarkBox, module_graph::ModuleGraph,
    structs::OutputChunk,
};

#[non_exhaustive]
pub struct Bundle {
    pub graph: ModuleGraph,
    pub output_options: Arc<BundleOptions>,
    pub mark_box: Arc<Mutex<MarkBox>>,
}

impl Bundle {
    pub fn new(
        graph: ModuleGraph,
        output_options: Arc<BundleOptions>,
        mark_box: Arc<Mutex<MarkBox>>,
    ) -> Self {
        Self {
            graph,
            output_options,
            mark_box,
        }
    }

    fn generate_chunks(&self) -> Vec<Chunk> {
        // TODO: code spliting
        let entries = DashSet::new();
        self.graph.node_idx_of_enties().iter().for_each(|entry| {
            let entry = self.graph.relation_graph[*entry].to_owned();
            entries.insert(entry);
        });

        let chunks = vec![Chunk {
            id: Default::default(),
            order_modules: self
                .graph
                .ordered_modules
                .clone()
                .into_iter()
                .map(|idx| self.graph.relation_graph[idx].clone())
                .collect(),
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
