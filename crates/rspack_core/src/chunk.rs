use crate::{Bundle, NormalizedBundleOptions};
use dashmap::DashMap;
use rayon::prelude::*;
use rspack_sources::{
  ConcatSource, GenMapOption, RawSource, Source, SourceMapSource, SourceMapSourceOptions,
};
use rspack_swc::swc::{self, TransformOutput};
use std::{
  collections::{hash_map::DefaultHasher, HashSet},
  hash::{Hash, Hasher},
  path::Path,
  sync::Arc,
};
use swc::Compiler;
use tracing::instrument;

#[derive(Debug)]
pub struct Chunk {
  pub id: String,
  pub kind: ChunkKind,
  pub filename: Option<String>,
  // pub order_modules: Vec<String>,
  pub entry_uri: String,
  pub module_ids: HashSet<String>,
  _noop: (),
}

impl Chunk {
  pub fn new(id: String, module_ids: HashSet<String>, entries: String, kind: ChunkKind) -> Self {
    Self {
      id,
      filename: Default::default(),
      module_ids,
      entry_uri: entries,
      // source_chunks: Default::default(),
      kind,
      _noop: (),
    }
  }

  pub fn from_js_module(id: String, module_id: String, kind: ChunkKind) -> Self {
    Self {
      id,
      filename: Default::default(),
      module_ids: Default::default(),
      entry_uri: module_id,
      // source_chunks: Default::default(),
      kind,
      _noop: (),
    }
  }

  #[instrument(skip_all)]
  pub fn render(
    &mut self,
    options: &NormalizedBundleOptions,
    compiler: Arc<Compiler>,
    bundle: &Bundle,
    output_modules: &DashMap<String, Arc<TransformOutput>>,
  ) -> OutputChunk {
    let mut concattables: Vec<Box<dyn Source>> = vec![];
    let modules = &bundle.module_graph.module_by_id;
    let mut module_ids = self.module_ids.iter().collect::<Vec<_>>();
    module_ids.sort_by_key(|id| 0 - modules[*id].exec_order);

    let mut not_transformed_module_ids: Vec<String> = vec![];
    for id in module_ids.iter().cloned() {
      if output_modules.get(id).is_none() && modules.get(id).is_some() {
        not_transformed_module_ids.push(id.clone());
      }
    }

    let rendered_modules = not_transformed_module_ids
      .par_iter()
      .map(|id| {
        let module = modules.get(id).unwrap();
        module.render(&compiler, modules, options, &bundle.context)
      })
      .collect::<Vec<_>>();

    for (id, output) in not_transformed_module_ids
      .iter()
      .zip(rendered_modules.into_iter())
    {
      output_modules.insert(id.to_string(), Arc::new(output));
    }

    module_ids.iter().cloned().for_each(|id| {
      if let Some(transform_output) = output_modules.get(id) {
        if let Some(map_string) = &transform_output.map.as_ref() {
          let source_map = sourcemap::SourceMap::from_slice(map_string.as_bytes()).unwrap();
          concattables.push(Box::new(SourceMapSource::new(SourceMapSourceOptions {
            source_code: transform_output.code.clone(),
            name: self.filename.as_ref().unwrap().clone(),
            source_map,
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          })));
        } else {
          concattables.push(Box::new(RawSource::new(&transform_output.code)));
        }
      }
    });

    let mut concat_source = ConcatSource::new(vec![]);
    concattables.iter_mut().for_each(|concattable| {
      concat_source.add(concattable.as_mut());
    });

    tracing::debug_span!("conncat_modules").in_scope(|| {
      let output_code = if options.source_map {
        let source_map_url = concat_source
          // FIXME: generate_url is slow now
          .generate_url(&GenMapOption {
            columns: true,
            include_source_contents: true,
            file: self.filename.clone().into(),
          })
          .unwrap()
          .unwrap();

        concat_source.source().to_string() + "\n//# sourceMappingURL=" + &source_map_url
      } else {
        concat_source.source().to_string()
      };

      OutputChunk {
        code: output_code,
        file_name: self.filename.as_ref().unwrap().clone(),
        entry: self.entry_uri.clone(),
      }
    })
  }

  pub fn get_chunk_info_with_file_names(&self) -> OutputChunk {
    OutputChunk {
      code: "".to_string(),
      file_name: self.filename.as_ref().unwrap().clone(),
      entry: self.entry_uri.clone(),
    }
  }

  #[inline]
  pub fn get_fallback_chunk_name(&self) -> &str {
    get_alias_name(&self.entry_uri)
  }

  #[inline]
  pub fn name(&self) -> &str {
    if let ChunkKind::Entry { name } = &self.kind {
      name.as_str()
    } else {
      "chunk"
    }
  }

  #[instrument()]
  pub fn generate_filename(&self, options: &NormalizedBundleOptions, bundle: &Bundle) -> String {
    let pendding_name = if self.kind.is_entry() {
      let pattern = &options.entry_filename;
      pattern
        .replace("[name]", self.name())
        .replace("[id]", &self.id)
    } else {
      let pattern = &options.chunk_filename;
      pattern.replace("[id]", &self.id)
    };

    match pendding_name.contains("contenthash") {
      true => {
        let content_hash = {
          let mut hasher = DefaultHasher::new();
          // FIXME: contenthash is not stable now.
          self.module_ids.iter().for_each(|module_id| {
            let module = &bundle.module_graph.module_by_id[module_id];
            module.ast.hash(&mut hasher);
          });
          hasher.finish()
        };
        pendding_name.replace("[contenthash]", &format!("{:x}", content_hash))
      }
      false => pendding_name,
    }
  }
}

#[inline]
fn get_alias_name(id: &str) -> &str {
  let p = Path::new(id);
  // +1 to include `.`
  let ext_len = p.extension().map_or(0, |s| s.to_string_lossy().len() + 1);
  let file_name = p.file_name().and_then(|name| name.to_str()).unwrap();
  &file_name[0..file_name.len() - ext_len]
}

#[derive(Debug)]
pub struct OutputChunk {
  pub code: String,
  pub file_name: String,
  pub entry: String,
}

#[derive(Debug)]
pub enum ChunkKind {
  Entry { name: String },
  Normal,
  // TODO: support it.
  // Initial,
}

impl ChunkKind {
  pub fn is_entry(&self) -> bool {
    matches!(self, ChunkKind::Entry { .. })
  }
  pub fn is_normal(&self) -> bool {
    matches!(self, ChunkKind::Normal)
  }
}
