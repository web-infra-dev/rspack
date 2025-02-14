use std::{cmp::Ordering, sync::Arc};

use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use rspack_core::{
  chunk_graph_chunk::ChunkId,
  rspack_sources::{ReplaceSource, Source},
  ChunkUkey, Compilation, CompilationAfterProcessAssets, CompilationAssets,
  CompilationProcessAssets, CrossOriginLoading,
};
use rspack_error::{Diagnostic, Result};
use rspack_hook::plugin_hook;
use rspack_plugin_real_content_hash::RealContentHashPluginUpdateHash;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{
  config::IntegrityHtmlPlugin,
  integrity::{compute_integrity, SubresourceIntegrityHashFunction},
  util::{make_placeholder, use_any_hash, PLACEHOLDER_PREFIX, PLACEHOLDER_REGEX},
  IntegrityCallbackData, SubresourceIntegrityPlugin, SubresourceIntegrityPluginInner,
};

#[derive(Debug, Clone)]
struct ProcessChunkResult {
  pub file: String,
  pub source: Option<Arc<dyn Source>>,
  pub warnings: Vec<String>,
  pub placeholder: Option<String>,
  pub integrity: Option<String>,
}

fn process_chunks(
  hash_funcs: &Vec<SubresourceIntegrityHashFunction>,
  compilation: &mut Compilation,
) -> HashMap<String, String> {
  let mut hash_by_placeholders = HashMap::default();
  let mut integrities = HashMap::default();
  let batches = digest_chunks(compilation);

  if matches!(
    compilation.options.output.cross_origin_loading,
    CrossOriginLoading::Disable
  ) {
    compilation.push_diagnostic(Diagnostic::warn(
      "SubresourceIntegrity".to_string(),
      r#"SRI requires a cross-origin policy, defaulting to "anonymous". 
Set rspack option output.crossOriginLoading to a value other than false 
to make this warning go away. 
See https://w3c.github.io/webappsec-subresource-integrity/#cross-origin-data-leakage"#
        .to_string(),
    ));
  }

  for batch in batches {
    let chunks = batch
      .into_iter()
      .filter_map(|c| compilation.chunk_by_ukey.get(&c))
      .collect::<Vec<_>>();

    let results = chunks
      .into_par_iter()
      .flat_map(|c| {
        let mut files = c
          .files()
          .iter()
          .map(|f| (c.id(&compilation.chunk_ids_artifact), f))
          .collect::<Vec<_>>();
        files.sort_by(|a, b| {
          let a_file = a.1.split("?").next().expect("should have a file name");
          let b_file = b.1.split("?").next().expect("should have a file name");
          if a_file.ends_with(".css") {
            Ordering::Less
          } else if b_file.ends_with(".css") {
            Ordering::Greater
          } else {
            a_file.cmp(b_file)
          }
        });
        files
      })
      .map(|(chunk_id, file)| {
        if let Some(source) = compilation.assets().get(file).and_then(|a| a.get_source()) {
          process_chunk_source(
            file,
            source.clone(),
            chunk_id,
            hash_funcs,
            &hash_by_placeholders,
          )
        } else {
          ProcessChunkResult {
            file: file.to_string(),
            source: None,
            warnings: vec![format!("No asset found for source path '{}'", file)],
            placeholder: None,
            integrity: None,
          }
        }
      })
      .collect::<Vec<_>>();

    let mut should_warn_content_hash = false;
    for result in results {
      for warning in result.warnings {
        compilation.push_diagnostic(Diagnostic::warn(
          "SubresourceIntegrity".to_string(),
          warning,
        ));
      }

      let Some(integrity) = result.integrity else {
        continue;
      };

      integrities.insert(result.file.clone(), integrity.clone());
      if let Some(placeholder) = result.placeholder {
        hash_by_placeholders.insert(placeholder, integrity.clone());
      }

      let real_content_hash = compilation.options.optimization.real_content_hash;

      if let Some(source) = result.source {
        if let Some(error) = compilation
          .update_asset(&result.file, |_, info| {
            if use_any_hash(&info) && (info.content_hash.is_empty() || !real_content_hash) {
              should_warn_content_hash = true;
            }

            let mut new_info = info.clone();
            new_info.content_hash.insert(integrity);
            Ok((Arc::new(source), new_info))
          })
          .err()
        {
          compilation.push_diagnostic(Diagnostic::error(
            "SubresourceIntegrity".to_string(),
            format!("Failed to update asset '{}': {}", result.file, error),
          ));
        }
      }
    }
    if should_warn_content_hash {
      compilation.push_diagnostic(Diagnostic::warn(
        "SubresourceIntegrity".to_string(),
        r#"Using [hash], [fullhash], [modulehash], or [chunkhash] is dangerous 
with SRI. The same is true for [contenthash] when realContentHash is disabled. 
Use [contenthash] and ensure realContentHash is enabled. See the README for 
more information."#
          .to_string(),
      ));
    }
  }

  integrities
}

fn process_chunk_source(
  file: &str,
  source: Arc<dyn Source>,
  chunk_id: Option<&ChunkId>,
  hash_funcs: &Vec<SubresourceIntegrityHashFunction>,
  hash_by_placeholders: &HashMap<String, String>,
) -> ProcessChunkResult {
  // generate new source
  let mut new_source = ReplaceSource::new(source.clone());

  let mut warnings = vec![];
  let source_content = source.source();
  if source_content.contains("webpackHotUpdate") {
    warnings.push("SubresourceIntegrity: SubResourceIntegrityPlugin may interfere with hot reloading. Consider disabling this plugin in development mode.".to_string());
  }

  // replace placeholders with integrity hash
  for caps in PLACEHOLDER_REGEX.captures_iter(&source_content) {
    if let Some(m) = caps.get(0) {
      let replacement = hash_by_placeholders
        .get(m.as_str())
        .map(|i| i.as_str())
        .unwrap_or(m.as_str());
      new_source.replace(m.start() as u32, m.end() as u32, replacement, None);
    }
  }

  // compute self integrity and placeholder
  let integrity = compute_integrity(hash_funcs, new_source.source().as_ref());
  let placeholder = chunk_id.map(|id| make_placeholder(hash_funcs, id.as_str()));

  ProcessChunkResult {
    file: file.to_string(),
    source: Some(Arc::new(new_source)),
    warnings,
    placeholder,
    integrity: Some(integrity),
  }
}

fn digest_chunks(compilation: &Compilation) -> Vec<HashSet<ChunkUkey>> {
  let mut batches = vec![];
  let mut visited_chunk_groups = HashSet::default();
  let mut visited_chunks = HashSet::default();
  let mut batch_chunk_groups = compilation.entrypoints().values().collect::<Vec<_>>();

  while !batch_chunk_groups.is_empty() {
    let mut chunk_batch = HashSet::default();
    for chunk_group in std::mem::take(&mut batch_chunk_groups) {
      if visited_chunk_groups.contains(chunk_group) {
        continue;
      }
      visited_chunk_groups.insert(chunk_group);
      if let Some(chunk_group) = compilation.chunk_group_by_ukey.get(chunk_group) {
        batch_chunk_groups.extend(chunk_group.children.iter());
        for chunk in chunk_group.chunks.iter() {
          if visited_chunks.contains(chunk) {
            continue;
          }
          visited_chunks.insert(*chunk);
          chunk_batch.insert(*chunk);
        }
      }
    }
    batches.push(chunk_batch);
  }
  batches.reverse();
  batches
}

fn add_minssing_integrities(
  assets: &CompilationAssets,
  integrities: &mut HashMap<String, String>,
  hash_func_names: &Vec<SubresourceIntegrityHashFunction>,
) {
  let new_integrities = assets
    .par_iter()
    .filter_map(|(src, asset)| {
      if integrities.contains_key(src) {
        return None;
      }
      asset.source.as_ref().map(|s| {
        let content = s.source();
        let integrity = compute_integrity(hash_func_names, &content);
        (src.clone(), integrity)
      })
    })
    .collect::<HashMap<_, _>>();

  integrities.extend(new_integrities);
}

#[plugin_hook(CompilationProcessAssets for SubresourceIntegrityPlugin, stage = Compilation::PROCESS_ASSETS_STAGE_OPTIMIZE_INLINE - 1)]
pub async fn handle_assets(&self, compilation: &mut Compilation) -> Result<()> {
  let integrities = process_chunks(&self.options.hash_func_names, compilation);
  let mut compilation_integrities =
    SubresourceIntegrityPlugin::get_compilation_integrities_mut(compilation.id());
  compilation_integrities.extend(integrities);

  if matches!(
    self.options.html_plugin,
    IntegrityHtmlPlugin::NativePlugin | IntegrityHtmlPlugin::JavaScriptPlugin
  ) {
    add_minssing_integrities(
      compilation.assets(),
      &mut compilation_integrities,
      &self.options.hash_func_names,
    );
  }

  if matches!(
    self.options.html_plugin,
    IntegrityHtmlPlugin::JavaScriptPlugin
  ) {
    if let Some(integrity_callback) = &self.options.integrity_callback {
      integrity_callback(IntegrityCallbackData {
        integerities: compilation_integrities.clone(),
      })?;
    }
  }

  Ok(())
}

#[plugin_hook(CompilationAfterProcessAssets for SubresourceIntegrityPlugin)]
pub async fn detect_unresolved_integrity(&self, compilation: &mut Compilation) -> Result<()> {
  let mut contain_unresolved_files = vec![];
  for chunk in compilation.chunk_by_ukey.values() {
    for file in chunk.files() {
      if let Some(source) = compilation.assets().get(file).and_then(|a| a.get_source()) {
        if source.source().contains(PLACEHOLDER_PREFIX.as_str()) {
          contain_unresolved_files.push(file.to_string());
        }
      }
    }
  }

  for file in contain_unresolved_files {
    compilation.push_diagnostic(Diagnostic::error(
      "SubresourceIntegrity".to_string(),
      format!("Asset {} contains unresolved integrity placeholders", file),
    ));
  }
  Ok(())
}

#[plugin_hook(RealContentHashPluginUpdateHash for SubresourceIntegrityPlugin)]
pub async fn update_hash(
  &self,
  compilation: &Compilation,
  assets: &[Arc<dyn Source>],
  old_hash: &str,
) -> Result<Option<String>> {
  let mut compilation_integrities =
    SubresourceIntegrityPlugin::get_compilation_integrities_mut(compilation.id());
  let key = compilation_integrities
    .iter()
    .filter_map(|(k, v)| {
      if v == old_hash {
        Some(k.to_string())
      } else {
        None
      }
    })
    .next();
  if let (Some(key), Some(asset)) = (key, assets.first()) {
    let content = asset.source();
    let new_integrity = compute_integrity(&self.options.hash_func_names, &content);
    compilation_integrities.insert(key, new_integrity.clone());
    return Ok(Some(new_integrity));
  }
  Ok(None)
}
