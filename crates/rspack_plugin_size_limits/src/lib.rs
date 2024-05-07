use std::fmt::Debug;

use derivative::Derivative;
use futures::future::BoxFuture;
use rspack_core::{
  ApplyContext, ChunkGroup, Compilation, CompilationAsset, CompilerAfterEmit, CompilerOptions,
  Plugin, PluginContext,
};
use rspack_error::{Diagnostic, Result};
use rspack_hook::{plugin, plugin_hook};
use rspack_util::size::format_size;

pub type AssetFilterFn = Box<dyn for<'a> Fn(&'a str) -> BoxFuture<'a, Result<bool>> + Sync + Send>;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct SizeLimitsPluginOptions {
  #[derivative(Debug = "ignore")]
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
      !asset.info.development
    }
  }

  async fn get_entrypoint_size(&self, entrypoint: &ChunkGroup, compilation: &Compilation) -> f64 {
    let mut size = 0.0;

    for filename in entrypoint.get_files(&compilation.chunk_by_ukey) {
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
    detail: &[(&String, f64)],
    limit: f64,
    hints: &str,
    diagnostics: &mut Vec<Diagnostic>,
  ) {
    let asset_list: String = detail
      .iter()
      .map(|&(name, size)| format!("\n  {} ({})", name, format_size(size)))
      .collect::<Vec<String>>()
      .join("");
    let title = String::from("assets over size limit warning");
    let message = format!("asset size limit: The following asset(s) exceed the recommended size limit ({}). This can impact web performance.\nAssets:{}", format_size(limit), asset_list);

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
            .map(|file| format!("      {}", file))
            .collect::<Vec<_>>()
            .join("\n")
        )
      })
      .collect::<Vec<_>>()
      .join("");
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
  let max_asset_size = self.options.max_asset_size.unwrap_or(250000.0);
  let max_entrypoint_size = self.options.max_entrypoint_size.unwrap_or(250000.0);

  let mut assets_over_size_limit = vec![];

  for (name, asset) in compilation.assets() {
    let source = asset.get_source();

    if !self.asset_filter(name, asset).await {
      continue;
    }

    if let Some(source) = source {
      let size = source.size() as f64;

      if size > max_asset_size {
        assets_over_size_limit.push((name, size));
      }
    }
  }

  let mut entrypoints_over_limit = vec![];

  for (name, ukey) in compilation.entrypoints.iter() {
    let entry = compilation.chunk_group_by_ukey.expect_get(ukey);
    let size = self.get_entrypoint_size(entry, compilation).await;

    if size > max_entrypoint_size {
      let mut files = vec![];

      for filename in entry.get_files(&compilation.chunk_by_ukey) {
        let asset = compilation.assets().get(&filename);

        if let Some(asset) = asset {
          if self.asset_filter(&filename, asset).await {
            files.push(filename);
          }
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
        max_asset_size,
        hints,
        &mut diagnostics,
      );
    }

    if !diagnostics.is_empty() {
      let has_async_chunk = compilation
        .chunk_by_ukey
        .values()
        .any(|chunk| !chunk.can_be_initial(&compilation.chunk_group_by_ukey));

      if !has_async_chunk {
        let title = String::from("no async chunks warning");
        let message = String::from("Rspack performance recommendations:\nYou can limit the size of your bundles by using import() to lazy load some parts of your application.\nFor more info visit https://www.rspack.dev/guide/optimization/code-splitting");

        Self::add_diagnostic(hints, title, message, &mut diagnostics);
      }

      compilation.push_batch_diagnostic(diagnostics);
    }
  }

  Ok(())
}

impl Plugin for SizeLimitsPlugin {
  fn name(&self) -> &'static str {
    "SizeLimitsPlugin"
  }

  fn apply(
    &self,
    ctx: PluginContext<&mut ApplyContext>,
    _options: &mut CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .compiler_hooks
      .after_emit
      .tap(after_emit::new(self));

    Ok(())
  }
}
