use anyhow::Context;
use rayon::prelude::{IntoParallelRefMutIterator, ParallelIterator};
use rspack_core::{parse_to_url, AssetContent, Plugin};
use serde::Deserialize;
use std::{fs, path::Path};
use swc_html::visit::VisitMutWith;

use crate::{
  config::{HtmlPluginConfig, HtmlPluginConfigInject},
  parser::HtmlCompiler,
  sri::{add_sri, create_digest_from_asset},
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
fn default_template() -> &'static str {
  r#"<!DOCTYPE html>
<html>
  <head>
    <meta charset="utf-8">
    <title>rspack</title>
  </head>
  <body>
  </body>
</html>"#
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
    let compilation = args.compilation;
    let assets = &compilation.assets;

    let _chunk_graph = &compilation.chunk_graph;
    let chunk_by_rid = &compilation.chunk_by_rid;

    let parser = HtmlCompiler::new();
    let (content, url) = match &config.template {
      Some(_template) => {
        let url = parse_to_url(_template);
        let resolved_template = resolve_from_context(&compilation.options.context, url.path());
        let content = fs::read_to_string(resolved_template).context(format!(
          "failed to read `{}` from `{}`",
          url.path(),
          &compilation.options.context
        ))?;
        (content, url)
      }
      None => (default_template().to_owned(), parse_to_url("default.html")),
    };

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
      .flat_map(|entry| entry.get_files(chunk_by_rid))
      .map(|asset_name| (asset_name.clone(), assets.get(&asset_name).unwrap()))
      .collect::<Vec<_>>();

    // entrypoint.get_files() are unstable, I need to sort it to pass tests.
    included_assets.sort_by(|(a, _), (b, _)| a.cmp(b));

    let mut tags = vec![];
    for (asset_name, asset) in included_assets {
      if let Some(extension) = Path::new(&asset_name).extension() {
        let mut asset_uri = asset_name.to_string();
        if let Some(public_path) = &config.public_path {
          asset_uri = format!("{}{}", public_path, asset_uri);
        }
        let mut tag: Option<HTMLPluginTag> = None;
        if extension.eq_ignore_ascii_case("css") {
          tag = Some(HTMLPluginTag::create_style(
            &asset_uri,
            Some(if let Some(inject) = &config.inject {
              *inject
            } else {
              HtmlPluginConfigInject::Head
            }),
          ));
        } else if extension.eq_ignore_ascii_case("js") || extension.eq_ignore_ascii_case("mjs") {
          tag = Some(HTMLPluginTag::create_script(
            &asset_uri,
            Some(if let Some(inject) = &config.inject {
              *inject
            } else if asset_uri.ends_with("runtime.js") {
              HtmlPluginConfigInject::Head
            } else {
              HtmlPluginConfigInject::Body
            }),
            &config.script_loading,
          ))
        }

        if let Some(tag) = tag {
          tags.push((tag, asset));
        }
      }
    }

    // if some plugin changes assets in the same stage after this plugin
    // both the name and the integrity may be inaccurate
    if let Some(hash_func) = &config.sri {
      tags.par_iter_mut().for_each(|(tag, asset)| {
        let sri_value = create_digest_from_asset(hash_func, asset);
        add_sri(tag, &sri_value);
      });
    }

    let tags = tags.into_iter().map(|(tag, _)| tag).collect::<Vec<_>>();
    let mut visitor = AssetWriter::new(config, &tags);
    current_ast.visit_mut_with(&mut visitor);

    let source = parser.codegen(&current_ast)?;
    compilation.emit_asset(
      config.filename.clone(),
      rspack_core::CompilationAsset {
        source: AssetContent::String(source),
      },
    );

    Ok(())
  }
}
