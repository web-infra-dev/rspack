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
        self.graph.resolved_entries.iter().for_each(|entry| {
            entries.insert(entry.id.clone());
        });

        let chunks = vec![Chunk {
            id: Default::default(),
            order_modules: self.graph.ordered_modules.clone(),
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
