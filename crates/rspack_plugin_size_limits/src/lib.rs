use derive_more::Debug;
use futures::future::BoxFuture;
use rspack_core::{
  CanonicalizedDataUrlOption, ChunkGroup, ChunkGroupUkey, Compilation, CompilationAsset,
  CompilerAfterEmit, Module, Plugin,
};
use rspack_error::{Diagnostic, Result, ToStringResultToRspackResultExt};
use rspack_hook::{plugin, plugin_hook};
use rspack_util::size::format_size;
use rustc_hash::FxHashMap as HashMap;

pub type AssetFilterFn = Box<dyn for<'a> Fn(&'a str) -> BoxFuture<'a, Result<bool>> + Sync + Send>;

#[derive(Debug)]
pub struct SizeLimitsPluginOptions {
  pub async_chunk_waterfalls: bool,
  #[debug(skip)]
  pub asset_filter: Option<AssetFilterFn>,
  pub embedded_source_maps: bool,
  pub hints: Option<String>,
  pub inlined_assets: bool,
  pub max_asset_size: Option<f64>,
  pub max_entrypoint_size: Option<f64>,
  pub top_level_this: bool,
}

#[plugin]
#[derive(Debug)]
pub struct SizeLimitsPlugin {
  options: SizeLimitsPluginOptions,
}

impl SizeLimitsPlugin {
  pub fn new(options: SizeLimitsPluginOptions) -> Self {
    Self::new_inner(options)
  }

  async fn asset_filter(&self, name: &str, asset: &CompilationAsset) -> bool {
    let asset_filter = &self.options.asset_filter;

    if let Some(asset_filter) = asset_filter {
      asset_filter(name)
        .await
        .expect("run SizeLimitsPlugin asset filter error")
    } else {
      !asset.info.development.unwrap_or(false)
    }
  }

  async fn get_entrypoint_size(&self, entrypoint: &ChunkGroup, compilation: &Compilation) -> f64 {
    let mut size = 0.0;

    for filename in entrypoint.get_files(&compilation.build_chunk_graph_artifact.chunk_by_ukey) {
      let asset = compilation.assets().get(&filename);

      if let Some(asset) = asset {
        if !self.asset_filter(&filename, asset).await {
          continue;
        }

        let source = asset.get_source();

        if let Some(source) = source {
          size += source.size() as f64;
        }
      }
    }

    size
  }

  fn add_diagnostic(
    hints: &str,
    title: String,
    message: String,
    diagnostics: &mut Vec<Diagnostic>,
  ) {
    let diagnostic = match hints {
      "error" => Diagnostic::error(title, message),
      "warning" => Diagnostic::warn(title, message),
      _ => Diagnostic::error(title, format!("Invalid hints type: {hints}")),
    };
    diagnostics.push(diagnostic);
  }

  fn add_assets_over_size_limit_warning(
    detail: &[(String, f64)],
    limit: f64,
    hints: &str,
    diagnostics: &mut Vec<Diagnostic>,
  ) {
    let asset_list: String = detail
      .iter()
      .map(|(name, size)| format!("\n  {} ({})", name, format_size(*size)))
      .collect::<String>();
    let title = String::from("assets over size limit warning");
    let message = format!(
      "asset size limit: The following asset(s) exceed the recommended size limit ({}). This can impact web performance.\nAssets:{}",
      format_size(limit),
      asset_list
    );

    Self::add_diagnostic(hints, title, message, diagnostics);
  }

  fn add_entrypoints_over_size_limit_warning(
    detail: &[(&String, f64, Vec<String>)],
    limit: f64,
    hints: &str,
    diagnostics: &mut Vec<Diagnostic>,
  ) {
    let entrypoint_list: String = detail
      .iter()
      .map(|(name, size, files)| {
        format!(
          "\n  {} ({})\n{}",
          name,
          format_size(*size),
          files
            .iter()
            .map(|file| format!("      {file}"))
            .collect::<Vec<_>>()
            .join("\n")
        )
      })
      .collect::<String>();
    let title = String::from("entrypoints over size limit warning");
    let message = format!(
      "entrypoint size limit: The following entrypoint(s) combined asset size exceeds the recommended limit ({}). This can impact web performance.\nEntrypoints:{}",
      format_size(limit),
      entrypoint_list
    );

    Self::add_diagnostic(hints, title, message, diagnostics);
  }

  fn chunk_group_name(compilation: &Compilation, group: &ChunkGroup) -> String {
    if let Some(name) = group.name() {
      return name.to_string();
    }

    group
      .chunks
      .first()
      .and_then(|ukey| {
        compilation
          .build_chunk_graph_artifact
          .chunk_by_ukey
          .get(ukey)
      })
      .and_then(|chunk| chunk.name().or_else(|| chunk.id().map(|id| id.as_str())))
      .unwrap_or("(unnamed)")
      .to_string()
  }

  fn async_chunk_waterfall_message(compilation: &Compilation) -> Option<String> {
    const MIN_REPORTED_DEPTH: usize = 3;
    const MAX_REPORTED_WATERFALLS: usize = 5;

    let groups = &compilation.build_chunk_graph_artifact.chunk_group_by_ukey;
    let mut paths: HashMap<ChunkGroupUkey, Vec<ChunkGroupUkey>> = HashMap::default();
    let mut queue = vec![];

    for (ukey, group) in groups {
      if group.is_initial() {
        paths.insert(*ukey, vec![]);
        queue.push(*ukey);
      }
    }

    let mut waterfalls = vec![];
    let mut deepest = 0;
    let mut index = 0;
    while index < queue.len() {
      let group = groups.expect_get(&queue[index]);
      let path = paths.expect_get(&queue[index]).clone();
      for child in group.children_iterable() {
        if paths.contains_key(child) {
          continue;
        }
        let mut child_path = path.clone();
        child_path.push(*child);
        paths.insert(*child, child_path.clone());
        queue.push(*child);

        let child_group = groups.expect_get(child);
        if child_path.len() < MIN_REPORTED_DEPTH || child_group.children_iterable().next().is_some()
        {
          continue;
        }

        let size = child_path
          .iter()
          .flat_map(|ukey| {
            groups
              .expect_get(ukey)
              .get_files(&compilation.build_chunk_graph_artifact.chunk_by_ukey)
          })
          .filter_map(|filename| compilation.assets().get(&filename))
          .filter_map(CompilationAsset::get_source)
          .map(|source| source.size())
          .sum::<usize>();
        deepest = deepest.max(child_path.len());
        waterfalls.push((
          child_path
            .iter()
            .map(|ukey| Self::chunk_group_name(compilation, groups.expect_get(ukey)))
            .collect::<Vec<_>>(),
          size,
        ));
      }
      index += 1;
    }

    if waterfalls.is_empty() {
      return None;
    }
    waterfalls.sort_by(|a, b| b.0.len().cmp(&a.0.len()).then_with(|| b.1.cmp(&a.1)));
    let details = waterfalls
      .iter()
      .take(MAX_REPORTED_WATERFALLS)
      .map(|(chain, size)| format!("\n  {} ({})", chain.join(" -> "), format_size(*size as f64)))
      .collect::<String>();
    Some(format!(
      "Async chunk waterfall: {} sequential async chunks are required before these leaves can load. Collapse nested import() calls or prefetch an earlier chunk.\nWaterfalls:{}",
      deepest, details
    ))
  }

  fn inlined_assets_message(compilation: &Compilation) -> Option<String> {
    const MAX_REPORTED_ASSETS: usize = 5;
    const DEFAULT_MAX_SIZE: usize = 8096;
    let mut assets = vec![];
    let mut total = 0;
    for (_, module) in compilation.get_module_graph().modules() {
      if !matches!(
        module
          .build_info()
          .asset
          .as_deref()
          .map(|asset| &asset.data_url),
        Some(CanonicalizedDataUrlOption::Asset(true))
      ) {
        continue;
      }
      let size = module.size(None, Some(compilation)).round() as usize;
      if size <= DEFAULT_MAX_SIZE {
        continue;
      }
      total += size;
      assets.push((
        module
          .readable_identifier(&compilation.options.context)
          .into_owned(),
        size,
      ));
    }
    if assets.is_empty() {
      return None;
    }
    assets.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let details = assets
      .iter()
      .take(MAX_REPORTED_ASSETS)
      .map(|(name, size)| format!("\n  {name} ({})", format_size(*size as f64)))
      .collect::<String>();
    Some(format!(
      "Inlined assets: {} asset module(s) larger than 8 KiB are embedded as data URLs ({} total). Consider asset/resource so browsers can cache them separately.\nAssets:{}",
      assets.len(),
      format_size(total as f64),
      details
    ))
  }

  fn top_level_this_message(compilation: &Compilation) -> Option<String> {
    const MAX_REPORTED_MODULES: usize = 5;
    let mut modules = vec![];
    let mut total = 0;
    for (_, module) in compilation.get_module_graph().modules() {
      let count = module.build_info().top_level_this;
      if count == 0 {
        continue;
      }
      total += count;
      modules.push((
        module
          .readable_identifier(&compilation.options.context)
          .into_owned(),
        count,
      ));
    }
    if modules.is_empty() {
      return None;
    }
    modules.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let details = modules
      .iter()
      .take(MAX_REPORTED_MODULES)
      .map(|(name, count)| format!("\n  {name} ({count} occurrence(s))"))
      .collect::<String>();
    Some(format!(
      "Top-level this: {} occurrence(s) in ES modules were replaced with undefined. Use imports, exports, or module exports explicitly instead.\nModules:{}",
      total, details
    ))
  }
}

#[plugin_hook(CompilerAfterEmit for SizeLimitsPlugin)]
async fn after_emit(&self, compilation: &mut Compilation) -> Result<()> {
  let hints = &self.options.hints;
  let max_asset_size = self.options.max_asset_size.unwrap_or(250_000.0);
  let max_entrypoint_size = self.options.max_entrypoint_size.unwrap_or(250_000.0);
  let mut checked_assets: HashMap<String, bool> = HashMap::default();
  let mut checked_chunk_groups: HashMap<ChunkGroupUkey, bool> = HashMap::default();

  let mut assets_over_size_limit = vec![];

  let asset_sizes = rspack_parallel::scope::<_, _>(|token| {
    compilation.assets().iter().for_each(|(name, asset)| {
      // SAFETY: await immediately and trust caller to poll future entirely
      let s = unsafe { token.used((&self, asset, name, max_asset_size)) };

      s.spawn(|(plugin, asset, name, max_asset_size)| async move {
        if !plugin.asset_filter(name, asset).await {
          return None;
        }

        let source = asset.get_source()?;

        let size = source.size() as f64;
        let is_over_size_limit = size > max_asset_size;
        Some((name.clone(), size, is_over_size_limit))
      })
    })
  })
  .await
  .into_iter()
  .map(|res| res.to_rspack_result())
  .collect::<Result<Vec<_>>>()?;

  for (name, size, is_over_size_limit) in asset_sizes.into_iter().flatten() {
    checked_assets.insert(name.clone(), is_over_size_limit);
    if is_over_size_limit {
      assets_over_size_limit.push((name, size));
    }
  }

  let mut entrypoints_over_limit = vec![];

  for (name, ukey) in compilation.build_chunk_graph_artifact.entrypoints.iter() {
    let entry = compilation
      .build_chunk_graph_artifact
      .chunk_group_by_ukey
      .expect_get(ukey);
    let size = self.get_entrypoint_size(entry, compilation).await;
    let is_over_size_limit = size > max_entrypoint_size;

    checked_chunk_groups.insert(ukey.to_owned(), is_over_size_limit);
    if is_over_size_limit {
      let mut files = vec![];

      for filename in entry.get_files(&compilation.build_chunk_graph_artifact.chunk_by_ukey) {
        let asset = compilation.assets().get(&filename);

        if let Some(asset) = asset
          && self.asset_filter(&filename, asset).await
        {
          files.push(filename);
        }
      }

      entrypoints_over_limit.push((name, size, files));
    }
  }

  if let Some(hints) = hints {
    let mut diagnostics = vec![];

    if !assets_over_size_limit.is_empty() {
      Self::add_assets_over_size_limit_warning(
        &assets_over_size_limit,
        max_asset_size,
        hints,
        &mut diagnostics,
      );
    }

    if !entrypoints_over_limit.is_empty() {
      Self::add_entrypoints_over_size_limit_warning(
        &entrypoints_over_limit,
        max_entrypoint_size,
        hints,
        &mut diagnostics,
      );
    }

    if self.options.async_chunk_waterfalls
      && let Some(message) = Self::async_chunk_waterfall_message(compilation)
    {
      Self::add_diagnostic(
        hints,
        "async chunk waterfalls warning".to_string(),
        message,
        &mut diagnostics,
      );
    }

    if self.options.embedded_source_maps {
      Self::add_diagnostic(
        hints,
        "embedded source maps warning".to_string(),
        "Embedded source maps increase every production JavaScript download. Use a separate source-map file or disable source maps for this build.".to_string(),
        &mut diagnostics,
      );
    }

    if self.options.inlined_assets
      && let Some(message) = Self::inlined_assets_message(compilation)
    {
      Self::add_diagnostic(
        hints,
        "inlined assets warning".to_string(),
        message,
        &mut diagnostics,
      );
    }

    if self.options.top_level_this
      && let Some(message) = Self::top_level_this_message(compilation)
    {
      Self::add_diagnostic(
        hints,
        "top-level this warning".to_string(),
        message,
        &mut diagnostics,
      );
    }

    if !diagnostics.is_empty() {
      let has_async_chunk = compilation
        .build_chunk_graph_artifact
        .chunk_by_ukey
        .values()
        .any(|chunk| {
          !chunk.can_be_initial(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey)
        });

      if !has_async_chunk {
        let title = String::from("no async chunks warning");
        let message = String::from(
          "Rspack performance recommendations:\nYou can limit the size of your bundles by using import() to lazy load some parts of your application.\nFor more info visit https://rspack.rs/guide/optimization/code-splitting",
        );

        Self::add_diagnostic(hints, title, message, &mut diagnostics);
      }

      compilation.extend_diagnostics(diagnostics);
    }
  }

  for (name, asset) in compilation.assets_mut() {
    if let Some(checked) = checked_assets.get(name) {
      asset.info.set_is_over_size_limit(*checked)
    }
  }

  for (ukey, checked) in checked_chunk_groups.iter() {
    compilation
      .build_chunk_graph_artifact
      .chunk_group_by_ukey
      .expect_get_mut(ukey)
      .set_is_over_size_limit(*checked);
  }

  Ok(())
}

impl Plugin for SizeLimitsPlugin {
  fn name(&self) -> &'static str {
    "SizeLimitsPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.after_emit.tap(after_emit::new(self));

    Ok(())
  }
}
