use std::collections::HashSet;
use std::sync::Arc;

use rspack_collections::DatabaseItem;
use rspack_core::{
  rspack_sources::{BoxSource, RawStringSource},
  ApplyContext, ChunkGraph, Compilation, CompilationAssets, CompilerEmit, CompilerOptions, Context,
  EntryDependency, Filename, LibIdentOptions, PathData, Plugin, PluginContext, ProvidedExports,
  SourceType,
};
use rspack_error::{Error, Result};
use rspack_hook::{plugin, plugin_hook};
use rustc_hash::FxHashMap as HashMap;

use crate::{
  DllManifest, DllManifestContent, DllManifestContentItem, DllManifestContentItemExports,
};

#[derive(Debug, Clone)]
pub struct LibManifestPluginOptions {
  pub context: Option<Context>,

  pub entry_only: Option<bool>,

  pub name: Option<Filename>,

  pub format: Option<bool>,

  pub path: Filename,

  pub r#type: Option<String>,
}

#[plugin]
#[derive(Debug)]
pub struct LibManifestPlugin {
  options: LibManifestPluginOptions,
}

impl LibManifestPlugin {
  pub fn new(options: LibManifestPluginOptions) -> Self {
    Self::new_inner(options)
  }
}

impl Plugin for LibManifestPlugin {
  fn name(&self) -> &'static str {
    "rspack.LibManifestPlugin"
  }

  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &CompilerOptions) -> Result<()> {
    ctx.context.compiler_hooks.emit.tap(emit::new(self));
    Ok(())
  }
}

#[plugin_hook(CompilerEmit for LibManifestPlugin)]
async fn emit(&self, compilation: &mut Compilation) -> Result<()> {
  let mut use_paths: HashSet<String> = HashSet::new();

  let chunk_graph = &compilation.chunk_graph;

  let mut manifests: CompilationAssets = HashMap::default();

  let module_graph = compilation.get_module_graph();

  for (_, chunk) in compilation.chunk_by_ukey.iter() {
    if !chunk.can_be_initial(&compilation.chunk_group_by_ukey) {
      continue;
    }

    let target_path = compilation.get_path(
      &self.options.path,
      PathData::default()
        .chunk_id_optional(chunk.id(&compilation.chunk_ids).map(|id| id.as_str()))
        .chunk_hash_optional(chunk.rendered_hash(
          &compilation.chunk_hashes_results,
          compilation.options.output.hash_digest_length,
        ))
        .chunk_name_optional(chunk.name_for_filename_template(&compilation.chunk_ids)),
    )?;

    if use_paths.contains(&target_path) {
      return Err(Error::msg("each chunk must have a unique path"));
    }

    use_paths.insert(target_path.clone());

    let name = self.options.name.as_ref().and_then(|name| {
      compilation
        .get_path(
          name,
          PathData::default()
            .chunk_id_optional(chunk.id(&compilation.chunk_ids).map(|id| id.as_str()))
            .chunk_hash_optional(chunk.rendered_hash(
              &compilation.chunk_hashes_results,
              compilation.options.output.hash_digest_length,
            ))
            .chunk_name_optional(chunk.name_for_filename_template(&compilation.chunk_ids))
            .content_hash_optional(chunk.rendered_content_hash_by_source_type(
              &compilation.chunk_hashes_results,
              &SourceType::JavaScript,
              compilation.options.output.hash_digest_length,
            )),
        )
        .ok()
    });

    let mut manifest_content: DllManifestContent = HashMap::default();

    for module in chunk_graph.get_ordered_chunk_modules(&chunk.ukey(), &module_graph) {
      if self.options.entry_only.unwrap_or_default()
        && !some_in_iterable(
          module_graph.get_incoming_connections(&module.identifier()),
          |conn| {
            let dep = module_graph.dependency_by_id(&conn.dependency_id);

            dep
              .map(|dep| dep.is::<EntryDependency>())
              .unwrap_or_default()
          },
        )
      {
        continue;
      }

      let context = match &self.options.context {
        Some(ctx) => ctx,
        None => &compilation.options.context,
      };

      let ident = module.lib_ident(LibIdentOptions { context });

      if let Some(ident) = ident {
        let exports_info = module_graph.get_exports_info(&module.identifier());

        let provided_exports = match exports_info.get_provided_exports(&module_graph) {
          ProvidedExports::Vec(vec) => Some(DllManifestContentItemExports::Vec(vec)),
          ProvidedExports::True => Some(DllManifestContentItemExports::True),
          _ => None,
        };

        let id = ChunkGraph::get_module_id(&compilation.module_ids, module.identifier());

        let build_meta = module.build_meta();

        manifest_content.insert(
          ident.into_owned(),
          DllManifestContentItem {
            id: id.map(|id| id.to_string()),
            build_meta: build_meta.cloned(),
            exports: provided_exports,
          },
        );
      }
    }

    let manifest = DllManifest {
      name,
      content: manifest_content,
      r#type: self.options.r#type.clone(),
    };

    let format = self.options.format.unwrap_or_default();

    let manifest_json = if format {
      serde_json::to_string_pretty(&manifest).map_err(|e| Error::msg(format!("{}", e)))?
    } else {
      serde_json::to_string(&manifest).map_err(|e| Error::msg(format!("{}", e)))?
    };

    let asset = Arc::new(RawStringSource::from(manifest_json)) as BoxSource;

    manifests.insert(target_path, asset.into());
  }

  for (filename, asset) in manifests {
    compilation.emit_asset(filename, asset);
  }

  Ok(())
}

fn some_in_iterable<I: Iterator, F>(iterable: I, filter: F) -> bool
where
  F: Fn(I::Item) -> bool,
{
  for item in iterable {
    if filter(item) {
      return true;
    }
  }
  false
}
