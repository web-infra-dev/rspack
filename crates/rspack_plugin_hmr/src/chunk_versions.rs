use std::{
  cmp::Ordering,
  sync::{Arc, Mutex, OnceLock},
};

use rspack_core::{
  ChunkGraph, ChunkKind, ChunkUkey, Compilation, RuntimeGlobals, RuntimeModule,
  RuntimeModuleGenerateContext, RuntimeModuleRuntimeRequirements, RuntimeModuleUpdateData,
  RuntimeTemplate, SourceType, chunk_graph_chunk::ChunkId, impl_runtime_module,
};
use smol_str::SmolStr;

#[derive(Debug)]
struct ChunkVersions(Box<[(ChunkId, SmolStr)]>);

impl RuntimeModuleUpdateData for ChunkVersions {}

#[derive(Debug, Default)]
struct CachedChunkVersions {
  versions: Option<Arc<ChunkVersions>>,
  generated: Option<(SmolStr, String)>,
}

fn chunk_js_version<'a>(compilation: &'a Compilation, chunk_ukey: &ChunkUkey) -> Option<&'a str> {
  compilation
    .chunk_hashes_artifact
    .get(chunk_ukey)
    .and_then(|hashes| hashes.content_hash().get(&SourceType::JavaScript))
    .map(|hash| hash.rendered(compilation.options.output.hash_digest_length))
}

fn push_json_str(buf: &mut String, s: &str) {
  if s
    .bytes()
    .all(|b| (b.is_ascii_graphic() || b == b' ') && b != b'"' && b != b'\\')
  {
    buf.push('"');
    buf.push_str(s);
    buf.push('"');
  } else {
    buf.push_str(&rspack_util::json_stringify_str(s));
  }
}

fn push_chunk_id_json(buf: &mut String, id: &ChunkId) {
  if id.as_number().is_some() {
    buf.push_str(id.as_str());
  } else {
    push_json_str(buf, id.as_str());
  }
}

fn push_version_assignment(buf: &mut String, map: &str, id: &ChunkId, version: &str) {
  buf.push_str(map);
  buf.push('[');
  push_chunk_id_json(buf, id);
  buf.push_str("] = ");
  push_json_str(buf, version);
  buf.push_str(";\n");
}

fn push_version_delete(buf: &mut String, map: &str, id: &ChunkId) {
  buf.push_str("delete ");
  buf.push_str(map);
  buf.push('[');
  push_chunk_id_json(buf, id);
  buf.push_str("];\n");
}

#[impl_runtime_module]
#[derive(Debug)]
pub struct HotModuleReplacementChunkVersionsRuntimeModule {
  #[cacheable(with=rspack_cacheable::with::Skip)]
  reachable_chunks: OnceLock<Box<[(ChunkId, ChunkUkey)]>>,
  #[cacheable(with=rspack_cacheable::with::Skip)]
  cache: Mutex<CachedChunkVersions>,
}

impl HotModuleReplacementChunkVersionsRuntimeModule {
  pub fn new(template: &RuntimeTemplate) -> Self {
    Self::with_default(template, Default::default(), Default::default())
  }

  fn versioned_chunks(&self, compilation: &Compilation) -> &[(ChunkId, ChunkUkey)] {
    self.reachable_chunks.get_or_init(|| {
      let graph = &compilation.build_chunk_graph_artifact;
      let root = graph
        .chunk_by_ukey
        .expect_get(&self.chunk().expect("should have runtime chunk"));
      let mut chunks: Vec<(ChunkId, ChunkUkey)> = root
        .get_all_referenced_chunks(&graph.chunk_group_by_ukey)
        .into_iter()
        .filter_map(|ukey| {
          let chunk = graph.chunk_by_ukey.expect_get(&ukey);
          // Runtime chunks receive `HMR_CHUNK_VERSIONS` in their tree runtime requirements,
          // so they must be excluded explicitly to avoid self-versioning.
          if matches!(chunk.kind(), ChunkKind::HotUpdate)
            || graph.is_runtime_chunk(chunk)
            || !ChunkGraph::get_chunk_runtime_requirements(compilation, &ukey)
              .contains(RuntimeGlobals::HMR_CHUNK_VERSIONS)
          {
            return None;
          }
          let id = chunk.id()?;
          Some((id.clone(), ukey))
        })
        .collect();
      chunks.sort_by(|a, b| a.0.cmp(&b.0));
      // Match BTreeMap::insert overwrite semantics if two chunks share an id: keep the later one.
      chunks.dedup_by(|later, earlier| {
        if later.0 == earlier.0 {
          earlier.1 = later.1;
          true
        } else {
          false
        }
      });
      chunks.into_boxed_slice()
    })
  }

  fn versions_match(
    compilation: &Compilation,
    chunks: &[(ChunkId, ChunkUkey)],
    cached: &[(ChunkId, SmolStr)],
  ) -> bool {
    let mut iter = chunks.iter().filter_map(|(id, ukey)| {
      let version = chunk_js_version(compilation, ukey)?;
      Some((id, version))
    });
    for (cached_id, cached_version) in cached {
      let Some((id, version)) = iter.next() else {
        return false;
      };
      if id != cached_id || version != cached_version.as_str() {
        return false;
      }
    }
    iter.next().is_none()
  }

  fn resolve_versions_locked<'a>(
    compilation: &Compilation,
    chunks: &[(ChunkId, ChunkUkey)],
    cached: &'a mut CachedChunkVersions,
  ) -> &'a Arc<ChunkVersions> {
    if cached
      .versions
      .as_ref()
      .is_some_and(|versions| Self::versions_match(compilation, chunks, &versions.0))
    {
      return cached.versions.as_ref().expect("checked above");
    }
    let entries: Box<[(ChunkId, SmolStr)]> = chunks
      .iter()
      .filter_map(|(id, ukey)| {
        let version = chunk_js_version(compilation, ukey)?;
        Some((id.clone(), SmolStr::new(version)))
      })
      .collect();
    cached.generated = None;
    cached.versions.insert(Arc::new(ChunkVersions(entries)))
  }

  fn versions(&self, compilation: &Compilation) -> Arc<ChunkVersions> {
    let chunks = self.versioned_chunks(compilation);
    let mut guard = self.cache.lock().expect("should lock chunk versions cache");
    Arc::clone(Self::resolve_versions_locked(
      compilation,
      chunks,
      &mut guard,
    ))
  }

  fn render_initial(map: &str, entries: &[(ChunkId, SmolStr)]) -> String {
    let mut out = String::with_capacity(96 + map.len() * 2 + entries.len() * 32);
    out.push_str(map);
    out.push_str(" = Object.create(null);\n[");
    for (idx, (id, version)) in entries.iter().enumerate() {
      if idx > 0 {
        out.push(',');
      }
      out.push('[');
      push_chunk_id_json(&mut out, id);
      out.push(',');
      push_json_str(&mut out, version);
      out.push(']');
    }
    out.push_str("].forEach(function(entry) { ");
    out.push_str(map);
    out.push_str("[entry[0]] = entry[1]; });");
    out
  }
}

#[async_trait::async_trait]
impl RuntimeModule for HotModuleReplacementChunkVersionsRuntimeModule {
  fn runtime_module_variables() -> &'static [&'static str] {
    &[]
  }

  fn dependent_hash(&self) -> bool {
    true
  }

  fn runtime_requirements(&self, _: &Compilation) -> RuntimeModuleRuntimeRequirements {
    RuntimeModuleRuntimeRequirements {
      // Updates mutate the existing map; the renderer owns its shared binding.
      dependencies: RuntimeGlobals::HMR_CHUNK_VERSIONS,
      ..Default::default()
    }
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let map = context
      .runtime_template
      .render_runtime_globals(&RuntimeGlobals::HMR_CHUNK_VERSIONS);
    let chunks = self.versioned_chunks(context.compilation);
    let mut guard = self.cache.lock().expect("should lock chunk versions cache");
    Self::resolve_versions_locked(context.compilation, chunks, &mut guard);
    if let Some((cached_map, cached_code)) = &guard.generated
      && cached_map.as_str() == map
    {
      return Ok(cached_code.clone());
    }
    let versions = guard.versions.as_ref().expect("resolved above");
    let rendered = Self::render_initial(&map, &versions.0);
    guard.generated = Some((SmolStr::new(&map), rendered.clone()));
    Ok(rendered)
  }

  fn get_update_data(&self, compilation: &Compilation) -> Option<Arc<dyn RuntimeModuleUpdateData>> {
    let data: Arc<dyn RuntimeModuleUpdateData> = self.versions(compilation);
    Some(data)
  }

  async fn generate_update(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
    previous: &dyn RuntimeModuleUpdateData,
  ) -> rspack_error::Result<Option<String>> {
    let Some(previous) = previous.as_any().downcast_ref::<ChunkVersions>() else {
      return Ok(None);
    };
    let versions = self.versions(context.compilation);
    let map = context
      .runtime_template
      .render_runtime_globals(&RuntimeGlobals::HMR_CHUNK_VERSIONS);
    let prev = &previous.0;
    let curr = &versions.0;
    let mut source = String::new();
    let mut deletes = String::new();
    let mut i = 0;
    let mut j = 0;
    while i < prev.len() && j < curr.len() {
      match prev[i].0.cmp(&curr[j].0) {
        Ordering::Less => {
          push_version_delete(&mut deletes, &map, &prev[i].0);
          i += 1;
        }
        Ordering::Greater => {
          push_version_assignment(&mut source, &map, &curr[j].0, &curr[j].1);
          j += 1;
        }
        Ordering::Equal => {
          if prev[i].1 != curr[j].1 {
            push_version_assignment(&mut source, &map, &curr[j].0, &curr[j].1);
          }
          i += 1;
          j += 1;
        }
      }
    }
    for (id, version) in &curr[j..] {
      push_version_assignment(&mut source, &map, id, version);
    }
    for (id, _) in &prev[i..] {
      push_version_delete(&mut deletes, &map, id);
    }
    source.push_str(&deletes);
    Ok(Some(source))
  }
}
