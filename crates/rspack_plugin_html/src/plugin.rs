use std::{
  collections::hash_map::DefaultHasher,
  fs,
  hash::{Hash, Hasher},
  path::{Path, PathBuf},
  sync::LazyLock,
};

use anyhow::Context;
use dojang::dojang::Dojang;
use itertools::Itertools;
use rayon::prelude::*;
use regex::Regex;
use rspack_core::{
  parse_to_url,
  rspack_sources::{RawSource, SourceExt},
  Compilation, CompilationAsset, CompilationProcessAssets, FilenameTemplate, PathData, Plugin,
};
use rspack_error::{miette, AnyhowError, Diagnostic, Result};
use rspack_hook::{plugin, plugin_hook};
use rspack_util::infallible::ResultInfallibleExt as _;
use swc_html::visit::VisitMutWith;

use crate::{
  config::{HtmlInject, HtmlRspackPluginOptions},
  parser::HtmlCompiler,
  sri::{add_sri, create_digest_from_asset},
  visitors::{
    asset::{AssetWriter, HTMLPluginTag},
    utils::{append_hash, generate_posix_path},
  },
};

static MATCH_DOJANG_FRAGMENT: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(r#"<%[-=]?\s*([\w.]+)\s*%>"#).expect("Failed to initialize `MATCH_DOJANG_FRAGMENT`")
});

#[plugin]
#[derive(Debug)]
pub struct HtmlRspackPlugin {
  config: HtmlRspackPluginOptions,
}

impl HtmlRspackPlugin {
  pub fn new(config: HtmlRspackPluginOptions) -> Result<Self> {
    Ok(Self::new_inner(config))
  }
}

#[plugin_hook(CompilationProcessAssets for HtmlRspackPlugin, stage = Compilation::PROCESS_ASSETS_STAGE_OPTIMIZE_INLINE)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  let config = &self.config;

  let mut error_content = vec![];

  let parser = HtmlCompiler::new(config);
  let (content, url, normalized_template_name) = if let Some(content) = &config.template_content {
    (
      content.clone(),
      parse_to_url("template_content.html").path().to_string(),
      "template_content.html".to_string(),
    )
  } else if let Some(template) = &config.template {
    // TODO: support loader query form
    let resolved_template = path_clean::clean(
      AsRef::<Path>::as_ref(&compilation.options.context).join(template.as_str()),
    );

    let content = fs::read_to_string(&resolved_template)
      .context(format!(
        "HtmlRspackPlugin: could not load file `{}` from `{}`",
        template, &compilation.options.context
      ))
      .map_err(AnyhowError::from);

    match content {
      Ok(content) => {
        let url = resolved_template.to_string_lossy().to_string();
        compilation.file_dependencies.insert(resolved_template);

        (content, url, template.clone())
      }
      Err(err) => {
        error_content.push(err.to_string());
        compilation.push_diagnostic(Diagnostic::from(miette::Error::from(err)));
        (
          default_template().to_owned(),
          parse_to_url("default.html").path().to_string(),
          template.clone(),
        )
      }
    }
  } else {
    (
      default_template().to_owned(),
      parse_to_url("default.html").path().to_string(),
      "default.html".to_string(),
    )
  };

  // process with template parameters
  let mut template_result = if let Some(template_parameters) = &self.config.template_parameters {
    let mut dj = Dojang::new();
    dj.add(url.clone(), content)
      .expect("failed to add template");
    dj.render(&url, serde_json::json!(template_parameters))
      .expect("failed to render template")
  } else {
    content
  };

  // dojang will not throw error when replace failed https://github.com/kev0960/dojang/issues/2
  if let Some(captures) = MATCH_DOJANG_FRAGMENT.captures(&template_result) {
    if let Some(name) = captures.get(1).map(|m| m.as_str()) {
      let error_msg = format!("ReferenceError: {name} is not defined");
      error_content.push(error_msg.clone());
      compilation.push_diagnostic(Diagnostic::from(miette::Error::msg(error_msg)));
    }
  }

  let has_doctype = template_result.contains("!DOCTYPE") || template_result.contains("!doctype");
  if !has_doctype {
    template_result = format!("<!DOCTYPE html>{template_result}");
  }

  let ast_with_diagnostic = parser.parse_file(&url, template_result)?;

  let (mut current_ast, diagnostic) = ast_with_diagnostic.split_into_parts();

  if !diagnostic.is_empty() {
    compilation.extend_diagnostics(diagnostic);
  }
  let included_assets = compilation
    .entrypoints
    .keys()
    .filter(|&entry_name| {
      let mut included = true;
      if let Some(included_chunks) = &config.chunks {
        included = included_chunks.iter().any(|c| c.eq(entry_name));
      }
      if let Some(exclude_chunks) = &config.exclude_chunks {
        included = included && !exclude_chunks.iter().any(|c| c.eq(entry_name));
      }
      included
    })
    .map(|entry_name| compilation.entrypoint_by_name(entry_name))
    .flat_map(|entry| entry.get_files(&compilation.chunk_by_ukey))
    .filter_map(|asset_name| {
      let asset = compilation.assets().get(&asset_name).expect("TODO:");
      if asset.info.hot_module_replacement || asset.info.development {
        None
      } else {
        Some((asset_name.clone(), asset))
      }
    })
    .collect::<Vec<_>>();

  let mut tags = vec![];
  // if inject is 'false', don't do anything
  if !matches!(config.inject, HtmlInject::False) {
    for (asset_name, asset) in included_assets {
      if let Some(extension) =
        Path::new(asset_name.split("?").next().unwrap_or_default()).extension()
      {
        let mut asset_uri = format!(
          "{}{}",
          config.get_public_path(compilation, &self.config.filename),
          url_encode_path(&asset_name)
        );
        if config.hash.unwrap_or_default() {
          if let Some(hash) = compilation.get_hash() {
            asset_uri = append_hash(&asset_uri, hash);
          }
        }
        let mut tag: Option<HTMLPluginTag> = None;
        if extension.eq_ignore_ascii_case("css") {
          tag = Some(HTMLPluginTag::create_style(
            &generate_posix_path(&asset_uri),
            HtmlInject::Head,
          ));
        } else if extension.eq_ignore_ascii_case("js") || extension.eq_ignore_ascii_case("mjs") {
          tag = Some(HTMLPluginTag::create_script(
            &generate_posix_path(&asset_uri),
            config.inject,
            &config.script_loading,
          ))
        }

        if let Some(tag) = tag {
          tags.push((tag, asset));
        }
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
  // Use the same filename as template
  let output_path = compilation
    .options
    .output
    .path
    .join(normalized_template_name);

  let html_file_name = FilenameTemplate::from(
    config
      .filename
      .replace("[templatehash]", "[contenthash]")
      .clone(),
  );
  // use to calculate relative favicon path when no publicPath
  let fake_html_file_name = compilation
    .get_path(
      &html_file_name,
      PathData::default().filename(&output_path.to_string_lossy()),
    )
    .always_ok();

  let mut visitor = AssetWriter::new(config, &tags, compilation, &fake_html_file_name);
  current_ast.visit_mut_with(&mut visitor);

  if let Some(favicon) = &self.config.favicon {
    let url = parse_to_url(favicon);
    let favicon_file_path = PathBuf::from(config.get_relative_path(compilation, favicon))
      .file_name()
      .expect("Should have favicon file name")
      .to_string_lossy()
      .to_string();

    let resolved_favicon = AsRef::<Path>::as_ref(&compilation.options.context).join(url.path());

    let content = fs::read(resolved_favicon)
      .context(format!(
        "HtmlRspackPlugin: could not load file `{}` from `{}`",
        favicon, &compilation.options.context
      ))
      .map_err(AnyhowError::from);

    match content {
      Ok(content) => {
        compilation.emit_asset(
          favicon_file_path,
          CompilationAsset::from(RawSource::from(content).boxed()),
        );
      }
      Err(err) => {
        error_content.push(err.to_string());
        compilation.push_diagnostic(Diagnostic::from(miette::Error::from(err)));
      }
    };
  }

  let mut source = if !error_content.is_empty() {
    format!(
      r#"Html Rspack Plugin:\n{}"#,
      error_content
        .iter()
        .map(|msg| format!(
          r#"
    <pre>
      Error: {msg}
    </pre>
    "#
        ))
        .join("\n")
    )
  } else {
    parser
      .codegen(&mut current_ast, compilation)?
      .replace("$$RSPACK_URL_AMP$$", "&")
  };

  if !has_doctype {
    source = source.replace("<!DOCTYPE html>", "");
  }
  let hash = hash_for_source(&source);

  let (output_path, asset_info) = compilation
    .get_path_with_info(
      &html_file_name,
      PathData::default()
        .filename(&output_path.to_string_lossy())
        .content_hash(&hash),
    )
    .always_ok();
  compilation.emit_asset(
    output_path,
    CompilationAsset::new(Some(RawSource::from(source).boxed()), asset_info),
  );

  Ok(())
}

impl Plugin for HtmlRspackPlugin {
  fn name(&self) -> &'static str {
    "rspack.HtmlRspackPlugin"
  }

  fn apply(
    &self,
    ctx: rspack_core::PluginContext<&mut rspack_core::ApplyContext>,
    _options: &mut rspack_core::CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .compilation_hooks
      .process_assets
      .tap(process_assets::new(self));
    Ok(())
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

fn hash_for_source(source: &str) -> String {
  let mut hasher = DefaultHasher::new();
  source.hash(&mut hasher);
  format!("{:016x}", hasher.finish())
}

fn url_encode_path(file_path: &str) -> String {
  let query_string_start = file_path.find('?');
  let url_path = if let Some(query_string_start) = query_string_start {
    &file_path[..query_string_start]
  } else {
    file_path
  };
  let query_string = if let Some(query_string_start) = query_string_start {
    &file_path[query_string_start..]
  } else {
    ""
  };

  format!(
    "{}{}",
    url_path
      .split('/')
      .map(|p| { urlencoding::encode(p) })
      .join("/"),
    // element.outerHTML will escape '&' so need to add a placeholder here
    query_string.replace("&", "$$RSPACK_URL_AMP$$")
  )
}
