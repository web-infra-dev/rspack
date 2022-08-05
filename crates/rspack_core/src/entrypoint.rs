use crate::ChunkGraph;

#[derive(Debug)]
pub struct Entrypoint {
  // pub(crate) files: Vec<String>,
  pub(crate) chunk_ids: Vec<String>,
}

// The implmentation is temporary and will be aligned with Webpack.
impl Entrypoint {
  pub fn new() -> Self {
    Self {
      // files,
      chunk_ids: Default::default(),
    }
  }

  /// the files contained in Entrypoint
  pub fn get_files(&self, chunk_graph: &ChunkGraph) -> Vec<String> {
    self
      .chunk_ids
      .iter()
      .flat_map(|chunk_id| {
        chunk_graph
          .chunk_by_id(chunk_id)
          .unwrap()
          .files
          .iter()
          .map(|file| file.to_string())
      })
      .collect::<Vec<_>>()
  }
}
