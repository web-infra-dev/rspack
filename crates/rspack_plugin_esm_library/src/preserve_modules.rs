use std::{
  path::{Path, PathBuf},
  sync::LazyLock,
};

use cow_utils::CowUtils;
use regex::Regex;
use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_core::{
  AssetGeneratorOptions, AssetResourceGeneratorOptions, Compilation, Filename, GeneratorOptions,
  Module, ModuleType, SourceType,
};
use rspack_util::fx_hash::{FxHashMap, FxHashSet};
use sugar_path::SugarPath;

use crate::EsmLibraryPlugin;

/// Matches the final `.ext` in a filename template so we can extract the
/// output extension (e.g. `.mjs` from `[name].mjs` or `dist/[name].js`).
static EXTENSION_RE: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r".+(\.[^.\[\]]+)$").expect("should compile extension regex"));

pub fn entry_modules(compilation: &Compilation) -> FxHashMap<String, IdentifierSet> {
  let module_graph = compilation.get_module_graph();
  compilation
    .entries
    .iter()
    .map(|(name, item)| {
      (
        name.clone(),
        item
          .all_dependencies()
          .chain(compilation.global_entry.all_dependencies())
          .filter_map(|dep_id| {
            // some entry dependencies may not find module because of resolve failed
            // so use filter_map to ignore them
            module_graph
              .module_identifier_by_dependency_id(dep_id)
              .copied()
          })
          .collect::<IdentifierSet>(),
      )
    })
    .collect()
}

pub fn entry_name_for_module(
  entry_modules: &FxHashMap<String, IdentifierSet>,
) -> IdentifierMap<FxHashSet<String>> {
  let mut entry_name_for_module: IdentifierMap<FxHashSet<String>> = IdentifierMap::default();
  for (entry_name, modules) in entry_modules {
    for module_id in modules {
      entry_name_for_module
        .entry(*module_id)
        .or_default()
        .insert(entry_name.clone());
    }
  }
  entry_name_for_module
}

/// Returns whether a module should be processed by `preserve_modules`.
/// JS, CSS and asset modules are relevant. Other types (wasm, etc.) are
/// handled by their own output pipelines.
fn should_preserve_module(
  module: &dyn Module,
  module_graph: &rspack_core::ModuleGraph,
  compilation: &Compilation,
) -> bool {
  let source_types = module.source_types(module_graph, Some(compilation));
  if source_types.iter().any(|t| {
    matches!(
      t,
      SourceType::JavaScript | SourceType::Css | SourceType::CssImport | SourceType::Asset
    )
  }) {
    return true;
  }
  // CSS modules created by the extract-css plugin use a `Custom` source type
  // (`css/mini-extract`). Detect them by their identifier prefix to keep
  // the dependency on `rspack_plugin_extract_css` out of this crate.
  module.identifier().starts_with("css|")
}

/// Whether this module produces a raw asset file (image, font, …). Asset
/// modules use their own filename template (`output.assetModuleFilename`)
/// rather than the chunk's `[name]` template, so `preserve_modules` has to
/// override it per-module instead of setting `chunk.name`.
fn is_asset_module(
  module: &dyn Module,
  module_graph: &rspack_core::ModuleGraph,
  compilation: &Compilation,
) -> bool {
  module
    .source_types(module_graph, Some(compilation))
    .iter()
    .any(|t| matches!(t, SourceType::Asset))
}

/// Whether this module contributes JavaScript output.
fn has_js_output(
  module: &dyn Module,
  module_graph: &rspack_core::ModuleGraph,
  compilation: &Compilation,
) -> bool {
  module
    .source_types(module_graph, Some(compilation))
    .iter()
    .any(|t| matches!(t, SourceType::JavaScript))
}

/// Returns the absolute path of a module's resource, supporting both
/// `NormalModule` and the synthetic `CssModule` from extract-css.
fn module_resource_path(module: &dyn Module) -> Option<PathBuf> {
  if let Some(normal_module) = module.as_normal_module() {
    return normal_module
      .resource_resolved_data()
      .path()
      .map(|p| p.as_std_path().to_path_buf());
  }
  // For non-normal modules (extract-css `CssModule`), derive the resource
  // path from `name_for_condition`, which strips the loader chain and query
  // string.
  let name = module.name_for_condition()?;
  let path = PathBuf::from(name.as_ref());
  if path.is_absolute() { Some(path) } else { None }
}

pub async fn preserve_modules(
  root: &Path,
  compilation: &mut Compilation,
) -> Vec<rspack_error::Diagnostic> {
  let mut errors = vec![];
  let modules = compilation
    .get_module_graph()
    .modules_keys()
    .copied()
    .collect::<Vec<_>>();

  let entry_modules = entry_modules(compilation);
  let entry_name_for_module = entry_name_for_module(&entry_modules);

  for module_id in modules {
    if compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .get_module_chunks(module_id)
      .is_empty()
    {
      // ignore orphan
      continue;
    }

    let (abs_path, is_asset, has_js) = {
      let module_graph = compilation.get_module_graph();
      let module = module_graph
        .module_by_identifier(&module_id)
        .expect("should have module");

      if !should_preserve_module(module.as_ref(), module_graph, compilation) {
        continue;
      }
      let Some(abs_path) = module_resource_path(module.as_ref()) else {
        continue;
      };
      (
        abs_path,
        is_asset_module(module.as_ref(), module_graph, compilation),
        has_js_output(module.as_ref(), module_graph, compilation),
      )
    };
    if !abs_path.starts_with(root) {
      continue;
    }

    // Compute the path relative to the `preserveModules` root.
    let file_path = abs_path.relative(root);
    // Normalise to forward slashes so chunk names (and the asset paths
    // derived from them) stay consistent across platforms. `to_slash()`
    // returns `None` for non-UTF8 paths — fall back to a lossy conversion
    // in that case rather than aborting the compilation.
    let file_path_str = match file_path.to_slash() {
      Some(s) => s.into_owned(),
      None => file_path
        .to_string_lossy()
        .cow_replace('\\', "/")
        .into_owned(),
    };

    // Asset modules (images, fonts, …) don't go through the chunk-level
    // filename template — their filename is computed by the asset plugin
    // from the module's own `generator.filename` or
    // `output.assetModuleFilename`. Override the module's asset filename
    // with the preserved source path (extension included) so the asset
    // emits at the right location AND any JS / CSS references to it pick
    // up the new path during code generation.
    if is_asset {
      let filename = Filename::from(file_path_str.clone());
      let module_graph = compilation.get_module_graph_mut();
      if let Some(module) = module_graph.module_by_identifier_mut(&module_id)
        && let Some(normal_module) = module.as_normal_module_mut()
      {
        // Try to mutate the existing generator options first — this preserves
        // any user-provided settings like `output_path` or `publicPath`.
        if let Some(opts) = normal_module.get_generator_options_mut()
          && opts.set_asset_filename(filename.clone())
        {
          continue;
        }
        // Otherwise initialise a minimal generator options whose variant
        // matches the module type, with just the filename set.
        let new_opts = match normal_module.module_type() {
          ModuleType::AssetResource => {
            GeneratorOptions::AssetResource(AssetResourceGeneratorOptions {
              filename: Some(filename),
              ..Default::default()
            })
          }
          _ => GeneratorOptions::Asset(AssetGeneratorOptions {
            filename: Some(filename),
            ..Default::default()
          }),
        };
        normal_module.set_generator_options(Some(new_opts));
      }
      continue;
    }

    let chunk = match EsmLibraryPlugin::get_module_chunk(module_id, compilation) {
      Ok(c) => c,
      Err(e) => {
        errors.push(e.into());
        continue;
      }
    };

    // Compute the `chunk.name` from the source path.
    //
    // Following Rollup's approach: strip the file extension and let the
    // per-type output template add the correct one back. rspack has separate
    // templates for each output type (`output.filename` → `[name].mjs` for
    // JS, `output.cssFilename` → `[name].css` for CSS, etc.), so we always
    // strip the source extension regardless of module type. This keeps
    // preserve_modules type-agnostic for naming.
    let base_name: String = if let Some(extension) = file_path.extension() {
      let suffix = format!(".{}", extension.to_string_lossy());
      file_path_str
        .strip_suffix(&suffix)
        .unwrap_or(&file_path_str)
        .to_string()
    } else {
      file_path_str
    };

    // Additionally, for chunks that will emit a JS file, pin a literal
    // `filename_template`. `chunk.name` alone only yields a unique output
    // when `output.filename` contains `[name]` / `[id]`; a fixed template
    // like `"bundle.mjs"` would otherwise collapse every preserved chunk
    // onto the same filename. The literal template bypasses that, matching
    // the pre-existing preserveModules guarantee.
    let js_filename_template: Option<Filename> = if has_js {
      let extension = compilation
        .build_chunk_graph_artifact
        .chunk_by_ukey
        .get(&chunk)
        .and_then(|c| c.filename_template().cloned())
        .unwrap_or_else(|| compilation.options.output.filename.clone());
      let ext = extension
        .template()
        .and_then(|tpl| EXTENSION_RE.captures(tpl).map(|c| c[1].to_string()))
        .unwrap_or_else(|| ".js".to_string());
      Some(Filename::from(format!("{base_name}{ext}")))
    } else {
      None
    };

    let entry_name = if let Some(entry_names) = entry_name_for_module.get(&module_id) {
      if entry_names.len() > 1 {
        errors.push(
          rspack_error::error!(
            "{} is used in multiple entries: [{}], this is not allowed in preserveModules",
            module_id,
            entry_names
              .iter()
              .map(|s| s.as_str())
              .collect::<Vec<_>>()
              .join(", ")
          )
          .into(),
        );
        continue;
      }

      entry_names.iter().next().cloned()
    } else {
      None
    };

    if compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .get_chunk_modules_identifier(&chunk)
      .len()
      == 1
    {
      // This is the last module in the chunk — rename in-place.
      let old_chunk = compilation
        .build_chunk_graph_artifact
        .chunk_by_ukey
        .expect_get_mut(&chunk);
      if let Some(old_name) = old_chunk.name().map(|s| s.to_string())
        && old_name != base_name
      {
        compilation
          .build_chunk_graph_artifact
          .named_chunks
          .remove(&old_name);
      }
      old_chunk.set_name(Some(base_name.clone()));
      if let Some(template) = js_filename_template {
        old_chunk.set_filename_template(Some(template));
      }
      compilation
        .build_chunk_graph_artifact
        .named_chunks
        .insert(base_name, chunk);
      continue;
    }

    let new_chunk_ukey =
      Compilation::add_chunk(&mut compilation.build_chunk_graph_artifact.chunk_by_ukey);
    compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .add_chunk(new_chunk_ukey);
    let [Some(new_chunk), Some(old_chunk)] = compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .get_many_mut([&new_chunk_ukey, &chunk])
    else {
      unreachable!("new_chunk and old_chunk should be inserted already")
    };

    new_chunk.set_name(Some(base_name.clone()));
    if let Some(template) = js_filename_template {
      new_chunk.set_filename_template(Some(template));
    }
    old_chunk.split(
      new_chunk,
      &mut compilation.build_chunk_graph_artifact.chunk_group_by_ukey,
    );
    compilation
      .build_chunk_graph_artifact
      .named_chunks
      .insert(base_name.clone(), new_chunk_ukey);

    // disconnect module from other chunks
    compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .disconnect_chunk_and_module(&chunk, module_id);

    compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .connect_chunk_and_module(new_chunk_ukey, module_id);

    if let Some(entry_name) = entry_name {
      compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .disconnect_chunk_and_entry_module(&chunk, module_id);

      let entrypoint = compilation.entrypoint_by_name_mut(&entry_name);
      let ukey = entrypoint.ukey;
      entrypoint.set_entrypoint_chunk(new_chunk_ukey);

      compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .connect_chunk_and_entry_module(new_chunk_ukey, module_id, ukey);

      // Remove the entry name from the old chunk to avoid filename conflicts.
      // Without this, the old chunk retains its entry name (e.g. "index") and
      // its output falls back to `[name].mjs` → "index.mjs", conflicting with
      // the new chunk that already owns that name.
      let old_chunk = compilation
        .build_chunk_graph_artifact
        .chunk_by_ukey
        .expect_get_mut(&chunk);
      if let Some(old_name) = old_chunk.name().map(|s| s.to_string()) {
        old_chunk.set_name(None);
        if old_name != base_name {
          compilation
            .build_chunk_graph_artifact
            .named_chunks
            .remove(&old_name);
        }
      }
    }
  }

  errors
}
