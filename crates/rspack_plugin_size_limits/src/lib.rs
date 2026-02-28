use std::collections::HashMap;

use derive_more::Debug;
use futures::future::BoxFuture;
use rspack_core::{
  ChunkGroup, ChunkGroupUkey, Compilation, CompilationAsset, CompilerAfterEmit, Plugin,
};
use rspack_error::{Diagnostic, Result, ToStringResultToRspackResultExt};
use rspack_hook::{plugin, plugin_hook};
use rspack_util::size::format_size;

pub type AssetFilterFn = Box<dyn for<'a> Fn(&'a str) -> BoxFuture<'a, Result<bool>> + Sync + Send>;

#[derive(Debug)]
pub struct SizeLimitsPluginOptions {
  #[debug(skip)]
  pub asset_filter: Option<AssetFilterFn>,
  pub hints: Option<String>,
  pub max_asset_size: Option<f64>,
  pub max_entrypoint_size: Option<f64>,
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
}

#[plugin_hook(CompilerAfterEmit for SizeLimitsPlugin)]
async fn after_emit(&self, compilation: &mut Compilation) -> Result<()> {
  let hints = &self.options.hints;
  let max_asset_size = self.options.max_asset_size.unwrap_or(250_000.0);
  let max_entrypoint_size = self.options.max_entrypoint_size.unwrap_or(250_000.0);
  let mut checked_assets: HashMap<String, bool> = HashMap::default();
  let mut checked_chunk_groups: HashMap<ChunkGroupUkey, bool> = HashMap::default();

  let mut assets_over_size_limit = vec![];

  let asset_sizes = rspack_futures::scope::<_, _>(|token| {
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
