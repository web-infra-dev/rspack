use crate::runtime::{rspack_runtime, RuntimeOptions};
use crate::{Bundle, NormalizedBundleOptions};
use rayon::prelude::*;
use rspack_sources::{
  ConcatSource, GenMapOption, RawSource, Source, SourceMapSource, SourceMapSourceOptions,
};
use rspack_swc::swc::{self};
use std::{
  collections::{hash_map::DefaultHasher, HashSet},
  hash::{Hash, Hasher},
  path::Path,
  sync::Arc,
};
use swc::Compiler;
use tracing::instrument;

#[derive(Debug)]
#[allow(clippy::manual_non_exhaustive)]
pub struct Chunk {
  pub id: String,
  pub kind: ChunkKind,
  pub filename: Option<String>,
  // pub order_modules: Vec<String>,
  pub entry_uri: String,
  pub module_uris: HashSet<String>,
  _noop: (),
}

impl Chunk {
  pub fn new(id: String, module_uris: HashSet<String>, entries: String, kind: ChunkKind) -> Self {
    Self {
      id,
      filename: Default::default(),
      module_uris,
      entry_uri: entries,
      // source_chunks: Default::default(),
      kind,
      _noop: (),
    }
  }

  pub fn from_js_module(id: String, module_uri: String, kind: ChunkKind) -> Self {
    Self {
      id,
      filename: Default::default(),
      module_uris: Default::default(),
      entry_uri: module_uri,
      // source_chunks: Default::default(),
      kind,
      _noop: (),
    }
  }

  #[instrument(skip_all)]
  pub fn render(&self, bundle: &Bundle) -> OutputChunk {
    let options = &bundle.context.options;
    let compiler = &bundle.context.compiler;

    let mut concattables: Vec<Box<dyn Source>> = vec![];
    let modules = &bundle.module_graph_container.module_graph;
    let mut module_uris = self.module_uris.iter().collect::<Vec<_>>();
    module_uris.sort_by_key(|id| 0 - modules.module_by_uri(*id).unwrap().exec_order);

    let rendered_modules = module_uris
      .par_iter()
      .map(|uri| {
        let module = modules.module_by_uri(uri).unwrap();
        module.render(compiler, modules, &bundle.context)
      })
      .collect::<Vec<_>>();
    if let ChunkKind::Entry { .. } = &self.kind {
      let code = rspack_runtime(&options.runtime);
      if code.trim() != "" {
        let runtime = Box::new(RawSource::new(&code));
        concattables.push(runtime);
      }
    }
    rendered_modules.iter().for_each(|transform_output| {
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
            file: self.filename.clone(),
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
          self.module_uris.iter().for_each(|module_uri| {
            let module = &bundle
              .module_graph_container
              .module_graph
              .module_by_uri(module_uri)
              .unwrap();
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
