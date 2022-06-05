// use crate::runtime::rspack_runtime;
use crate::{Bundle, NormalizedBundleOptions, Platform, RuntimeInjector, SourceMapOptions};
use rayon::prelude::*;
use rspack_sources::{
  ConcatSource, GenMapOption, RawSource, Source, SourceMapSource, SourceMapSourceOptions,
};
use std::{collections::HashSet, path::Path};
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

    let mut concattables: Vec<Box<dyn Source>> = vec![];
    let modules = &bundle.module_graph_container.module_graph;
    let mut module_uris = self.module_uris.iter().collect::<Vec<_>>();
    module_uris.sort_by_key(|id| 0 - modules.module_by_uri(*id).unwrap().exec_order);

    let rendered_modules = module_uris
      .par_iter()
      .map(|uri| {
        let module = modules.module_by_uri(uri).unwrap();
        module.render(bundle)
      })
      .collect::<Vec<_>>();

    if let ChunkKind::Entry { .. } = &self.kind {
      // this should be globalized
      let injector = RuntimeInjector {
        cjs_runtime_hmr: options.runtime.hmr.into(),
        cjs_runtime_browser: matches!(options.platform, Platform::Browser).into(),
        cjs_runtime_node: matches!(options.platform, Platform::Node).into(),
        ..Default::default()
      };

      let code = injector.as_code(bundle).unwrap();

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
      let gen_map_option = &GenMapOption {
        columns: true,
        include_source_contents: true,
        file: self.filename.clone(),
      };
      let output_chunk = match options.source_map {
        SourceMapOptions::Inline => {
          let map = concat_source.generate_url(gen_map_option).unwrap().unwrap();
          (concat_source.source(), OutputChunkSourceMap::Inline(map))
        }
        SourceMapOptions::External => {
          let map = concat_source
            .generate_string(gen_map_option)
            .unwrap()
            .unwrap();
          (concat_source.source(), OutputChunkSourceMap::External(map))
        }
        SourceMapOptions::Linked => {
          let map = concat_source
            .generate_string(gen_map_option)
            .unwrap()
            .unwrap();
          (concat_source.source(), OutputChunkSourceMap::Linked(map))
        }
        SourceMapOptions::None => (concat_source.source(), OutputChunkSourceMap::None),
      };

      OutputChunk {
        code: output_chunk.0.into(),
        file_name: self.filename.as_ref().unwrap().clone(),
        entry: self.entry_uri.clone(),
        map: output_chunk.1,
      }
    })
  }

  pub fn get_chunk_info_with_file_names(&self) -> OutputChunk {
    OutputChunk {
      code: "".to_string(),
      file_name: self.filename.as_ref().unwrap().clone(),
      entry: self.entry_uri.clone(),
      map: OutputChunkSourceMap::None,
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
  /// Currently we defer calc `[contenthash]` until render output chunk
  pub fn generate_filename(&self, options: &NormalizedBundleOptions, bundle: &Bundle) -> String {
    let pending_name = if self.kind.is_entry() {
      let pattern = &options.entry_filename;
      pattern
        .replace("[name]", self.name())
        .replace("[id]", &self.id)
    } else {
      let pattern = &options.chunk_filename;
      pattern.replace("[id]", &self.id)
    };
    pending_name
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

#[derive(Debug, Clone)]
pub enum OutputChunkSourceMap {
  Inline(String),
  External(String),
  Linked(String),
  None,
}

impl OutputChunkSourceMap {
  pub fn is_exist(&self) -> bool {
    !matches!(self, OutputChunkSourceMap::None)
  }
}

#[derive(Debug, Clone)]
pub struct OutputChunk {
  pub code: String,
  pub file_name: String,
  pub entry: String,
  pub map: OutputChunkSourceMap,
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
