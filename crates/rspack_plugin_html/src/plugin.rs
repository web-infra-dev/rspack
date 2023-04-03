use std::{fs, path::Path};

use anyhow::Context;
use async_trait::async_trait;
use dojang::dojang::Dojang;
use rayon::prelude::{IntoParallelRefMutIterator, ParallelIterator};
use rspack_core::{
  parse_to_url,
  rspack_sources::{RawSource, SourceExt},
  CompilationAsset, Plugin,
};
use serde::Deserialize;
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

#[async_trait]
impl Plugin for HtmlPlugin {
  fn name(&self) -> &'static str {
    "html"
  }

  async fn process_assets_stage_optimize_inline(
    &mut self,
    _ctx: rspack_core::PluginContext,
    args: rspack_core::ProcessAssetsArgs<'_>,
  ) -> rspack_core::PluginProcessAssetsOutput {
    let config = &self.config;
    let compilation = args.compilation;

    let parser = HtmlCompiler::new(config);
    let content = if let Some(content) = &config.template_content {
      content.clone()
    } else if let Some(template) = &config.template {
      // TODO: support loader query form
      let resolved_template = resolve_from_context(&compilation.options.context, template.as_str());

      fs::read_to_string(&resolved_template).context(format!(
        "failed to read `{}` from `{}`",
        resolved_template.display(),
        &compilation.options.context.display()
      ))?
    } else {
      default_template().to_owned()
    };

    // process with template parameters
    let template_result = if let Some(template_parameters) = &self.config.template_parameters {
      let mut dj = Dojang::new();
      let temp_name = String::from("temp");
      dj.add(temp_name.clone(), content)
        .expect("failed to add template");
      dj.render(&temp_name, serde_json::json!(template_parameters))
        .expect("failed to render template")
    } else {
      content
    };

    let ast_with_diagnostic = parser.parse_file("builtin.html", template_result)?;

    let (mut current_ast, diagnostic) = ast_with_diagnostic.split_into_parts();

    if !diagnostic.is_empty() {
      compilation.push_batch_diagnostic(diagnostic);
    }
    let included_assets = compilation
      .entrypoints
      .keys()
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
      .map(|entry_name| compilation.entrypoint_by_name(entry_name))
      .flat_map(|entry| entry.get_files(&compilation.chunk_by_ukey))
      .map(|asset_name| {
        (
          asset_name.clone(),
          compilation.assets.get(&asset_name).expect("TODO:"),
        )
      })
      .collect::<Vec<_>>();

    let mut tags = vec![];
    for (asset_name, asset) in included_assets {
      if let Some(extension) = Path::new(&asset_name).extension() {
        let asset_uri = format!(
          "{}{asset_name}",
          config.get_public_path(compilation, &asset_name),
        );
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
      tags
        .par_iter_mut()
        .filter_map(|(tag, asset)| asset.get_source().map(|s| (tag, s)))
        .for_each(|(tag, asset)| {
          let sri_value = create_digest_from_asset(hash_func, asset);
          add_sri(tag, &sri_value);
        });
    }

    let tags = tags.into_iter().map(|(tag, _)| tag).collect::<Vec<_>>();
    let mut visitor = AssetWriter::new(config, &tags, compilation);
    current_ast.visit_mut_with(&mut visitor);

    let source = parser.codegen(&mut current_ast)?;
    compilation.emit_asset(
      config.filename.clone(),
      CompilationAsset::with_source(RawSource::from(source).boxed()),
    );

    if let Some(favicon) = &self.config.favicon {
      let url = parse_to_url(favicon);
      let resolved_favicon = resolve_from_context(&compilation.options.context, url.path());
      let content = fs::read(resolved_favicon).context(format!(
        "failed to read `{}` from `{}`",
        url.path(),
        &compilation.options.context.display()
      ))?;
      compilation.emit_asset(
        favicon.clone(),
        CompilationAsset::with_source(RawSource::from(content).boxed()),
      );
    }

    Ok(())
  }
}
