use anyhow::Context;
use rspack_core::{parse_to_url, AssetContent, Plugin};
use serde::Deserialize;
use std::{fs, path::Path};
use swc_html::visit::VisitMutWith;

use crate::{
  config::HtmlPluginConfig,
  parser::HtmlCompiler,
  utils::resolve_from_context,
  visitors::asset::{AssetWriter, HTMLPluginTag},
};

#[derive(Deserialize, Debug, Default)]
pub struct HtmlPlugin {
  config: HtmlPluginConfig,
}

impl HtmlPlugin {
  pub fn new(config: HtmlPluginConfig) -> HtmlPlugin {
    HtmlPlugin { config }
  }
}

impl Plugin for HtmlPlugin {
  fn name(&self) -> &'static str {
    "html"
  }

  fn process_assets(
    &self,
    _ctx: rspack_core::PluginContext,
    args: rspack_core::ProcessAssetsArgs,
  ) -> rspack_core::PluginProcessAssetsOutput {
    let config = &self.config;
    let mut compilation = args.compilation.borrow_mut();
    let assets = &compilation.assets;

    let chunk_graph = &compilation.chunk_graph;

    let parser = HtmlCompiler::new();

    let url = parse_to_url(&config.template);
    let resolved_template = resolve_from_context(&compilation.options.context, url.path());
    let content = fs::read_to_string(resolved_template).context(format!(
      "while reading {} from {:?}",
      url.path(),
      std::env::current_dir().unwrap_or_default()
    ))?;

    let mut current_ast = parser.parse_file(url.path(), content)?;

    let mut included_assets = compilation
      .entrypoints
      .keys()
      .into_iter()
      .filter(|&entry_name| {
        let mut included = true;
        if let Some(included_chunks) = &config.chunks {
          included = included_chunks.iter().any(|c| c.eq(entry_name));
        }
        if let Some(excluded_chunks) = &config.excluded_chunks {
          included = included && !excluded_chunks.iter().any(|c| c.eq(entry_name));
        }
        included
      })
      .map(|entry_name| compilation.entrypoints.get(entry_name).unwrap())
      .flat_map(|entry| entry.get_files(chunk_graph))
      .map(|asset_name| (asset_name.clone(), assets.get(&asset_name).unwrap()))
      .collect::<Vec<_>>();

    // entrypoint.get_files() are unstable, I need to sort it to pass tests.
    included_assets.sort_by(|(a, _), (b, _)| a.cmp(b));

    let mut tags = vec![];
    for (asset_name, _asset) in included_assets {
      if let Some(extension) = Path::new(&asset_name).extension() {
        let mut asset_uri = asset_name.to_string();
        if let Some(public_path) = &config.public_path {
          asset_uri = format!("{}{}", public_path, asset_uri);
        }
        if extension.eq_ignore_ascii_case("css") {
          tags.push(HTMLPluginTag::create_style(
            &asset_uri,
            Some(if let Some(inject) = &config.inject {
              inject.clone()
            } else {
              "head".to_string()
            }),
          ))
        } else if extension.eq_ignore_ascii_case("js") || extension.eq_ignore_ascii_case("mjs") {
          tags.push(HTMLPluginTag::create_script(
            &asset_uri,
            Some(if let Some(inject) = &config.inject {
              inject.clone()
            } else {
              "body".to_string()
            }),
            &config.script_loading,
          ))
        }
      }
    }

    let mut visitor = AssetWriter::new(config, &tags);
    current_ast.visit_mut_with(&mut visitor);

    if let Ok(source) = parser.codegen(&current_ast) {
      compilation.emit_asset(
        config.filename.clone(),
        rspack_core::CompilationAsset {
          source: AssetContent::String(source),
        },
      );
    }

    Ok(())
  }
}
