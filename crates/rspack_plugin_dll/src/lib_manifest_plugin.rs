use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use rspack_core::{
  rspack_sources::{BoxSource, RawSource},
  ApplyContext, BuildMeta, Compilation, CompilerEmit, CompilerOptions, Context, EntryDependency,
  Filename, LibIdentOptions, PathData, Plugin, PluginContext, ProvidedExports, SourceType,
};
use rspack_error::{Error, Result};
use rspack_hook::{plugin, plugin_hook};
use rspack_util::atom::Atom;
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct LibManifestPluginOptions {
  pub context: Option<Context>,

  pub entry_only: Option<bool>,

  pub name: Option<Filename>,

  pub format: Option<bool>,

  pub path: Filename,

  pub ty: Option<String>,
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

  let mut manifest_files = vec![];

  let module_graph = compilation.get_module_graph();

  for (_, chunk) in compilation.chunk_by_ukey.iter() {
    if !chunk.can_be_initial(&compilation.chunk_group_by_ukey) {
      return Ok(());
    }

    let target_path = compilation.get_path(
      &self.options.path,
      PathData {
        chunk: Some(chunk),
        ..Default::default()
      },
    )?;

    if use_paths.get(&target_path).is_some() {
      return Err(Error::msg("each chunk must have a unique path"));
    }

    use_paths.insert(target_path.clone());

    let name = self.options.name.as_ref().and_then(|filename| {
      compilation
        .get_path(
          filename,
          PathData {
            chunk: Some(chunk),
            content_hash_type: Some(SourceType::JavaScript),
            ..Default::default()
          },
        )
        .ok()
    });

    let mut manifest_contents = HashMap::<Cow<str>, ManifestContent>::new();

    for module in chunk_graph.get_ordered_chunk_modules(&chunk.ukey, &module_graph) {
      if self.options.entry_only.unwrap_or_default()
        && !some_in_iterable(
          module_graph
            .get_incoming_connections(&module.identifier())
            .into_iter(),
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
          ProvidedExports::Vec(vec) => Some(vec),
          _ => None,
        };

        let id = chunk_graph.get_module_id(module.identifier());

        let build_meta = module.build_meta();

        manifest_contents.insert(
          ident,
          ManifestContent {
            id,
            build_meta,
            provided_exports,
          },
        );
      }
    }

    let manifest = Manifest {
      name: name.as_deref(),
      content: manifest_contents,
      ty: self.options.ty.clone(),
    };

    let format = self.options.format.unwrap_or_default();

    let manifest_json = if format {
      serde_json::to_string_pretty(&manifest).map_err(|e| Error::msg(format!("{}", e)))?
    } else {
      serde_json::to_string(&manifest).map_err(|e| Error::msg(format!("{}", e)))?
    };

    manifest_files.push(ManifestFile {
      filename: target_path.clone(),
      content: manifest_json.clone(),
    });
  }

  for file in manifest_files {
    let filename = file.filename;
    let manifest_content = file.content;
    let manifest_asset = Arc::new(RawSource::from(manifest_content)) as BoxSource;

    compilation.emit_asset(filename, manifest_asset.into());
  }

  Ok(())
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ManifestContent<'i> {
  #[serde(skip_serializing_if = "Option::is_none")]
  id: Option<&'i str>,

  #[serde(skip_serializing_if = "Option::is_none")]
  build_meta: Option<&'i BuildMeta>,

  #[serde(skip_serializing_if = "Option::is_none")]
  provided_exports: Option<Vec<Atom>>,
}

#[derive(Serialize, Debug)]
struct Manifest<'i> {
  #[serde(skip_serializing_if = "Option::is_none")]
  name: Option<&'i str>,

  content: HashMap<Cow<'i, str>, ManifestContent<'i>>,

  #[serde(rename = "type")]
  #[serde(skip_serializing_if = "Option::is_none")]
  ty: Option<String>,
}

#[derive(Debug)]
struct ManifestFile {
  filename: String,

  content: String,
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
  return false;
}
