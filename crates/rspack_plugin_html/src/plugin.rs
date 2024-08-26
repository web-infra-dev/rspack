use std::{
  collections::{hash_map::DefaultHasher, HashMap},
  fs,
  hash::{Hash, Hasher},
  path::{Path, PathBuf},
};

use anyhow::Context;
use itertools::Itertools;
use rayon::prelude::*;
use rspack_core::{
  parse_to_url,
  rspack_sources::{RawSource, SourceExt},
  Compilation, CompilationAsset, CompilationProcessAssets, CrossOriginLoading, FilenameTemplate,
  Mode, PathData, Plugin,
};
use rspack_dojang::dojang::{Dojang, DojangOptions};
use rspack_dojang::Operand;
use rspack_error::{miette, AnyhowError, Diagnostic, Result};
use rspack_hook::{plugin, plugin_hook};
use rspack_paths::AssertUtf8;
use rspack_util::infallible::ResultInfallibleExt as _;
<<<<<<< HEAD
use serde_json::Value;
=======
>>>>>>> 882c65472 (feat(html): improve template parameters)
use sugar_path::SugarPath;
use swc_html::visit::VisitMutWith;

use crate::{
  config::{HtmlInject, HtmlRspackPluginOptions, HtmlScriptLoading},
  parser::HtmlCompiler,
  sri::{add_sri, create_digest_from_asset},
  visitors::{
    asset::AssetWriter,
    tag::HTMLPluginTag,
    utils::{append_hash, generate_posix_path, html_tag_object_to_string, merge_json},
  },
};

#[plugin]
#[derive(Debug)]
pub struct HtmlRspackPlugin {
  config: HtmlRspackPluginOptions,
}

impl HtmlRspackPlugin {
  pub fn new(config: HtmlRspackPluginOptions) -> Self {
    Self::new_inner(config)
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
      compilation
        .options
        .context
        .as_path()
        .join(template.as_str()),
    )
    .assert_utf8();

    let content = fs::read_to_string(&resolved_template)
      .context(format!(
        "HtmlRspackPlugin: could not load file `{}` from `{}`",
        template, &compilation.options.context
      ))
      .map_err(AnyhowError::from);

    match content {
      Ok(content) => {
        let url = resolved_template.as_str().to_string();
        compilation
          .file_dependencies
          .insert(resolved_template.into_std_path_buf());

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
    let default_src_template =
      path_clean::clean(compilation.options.context.as_path().join("src/index.ejs")).assert_utf8();

    if let Ok(content) = fs::read_to_string(&default_src_template) {
      let url = default_src_template.as_str().to_string();
      compilation
        .file_dependencies
        .insert(default_src_template.into_std_path_buf());

      (content, url, "src/index.ejs".to_string())
    } else {
      (
        default_template().to_owned(),
        parse_to_url("default.html").path().to_string(),
        "default.html".to_string(),
      )
    }
  };

  let mut asset_tags = vec![];

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

  let mut assets = HashMap::<String, Vec<String>>::default();

  // if inject is 'false', don't do anything
  for (asset_name, asset) in included_assets {
    if let Some(extension) = Path::new(asset_name.split("?").next().unwrap_or_default()).extension()
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
      let final_path = generate_posix_path(&asset_uri);
      if extension.eq_ignore_ascii_case("css") {
        assets
          .entry("css".to_string())
          .or_default()
          .push(final_path.to_string());
        tag = Some(HTMLPluginTag::create_style(
          &generate_posix_path(&asset_uri),
          HtmlInject::Head,
        ));
      } else if extension.eq_ignore_ascii_case("js") || extension.eq_ignore_ascii_case("mjs") {
        assets
          .entry("js".to_string())
          .or_default()
          .push(final_path.to_string());
        tag = Some(HTMLPluginTag::create_script(
          &generate_posix_path(&asset_uri),
          config.inject,
          &config.script_loading,
        ));
      }

      if let Some(tag) = tag {
        asset_tags.push((tag, asset));
      }
    }
  }

  // if some plugin changes assets in the same stage after this plugin
  // both the name and the integrity may be inaccurate
  if let Some(hash_func) = &config.sri {
    asset_tags
      .par_iter_mut()
      .filter_map(|(tag, asset)| asset.get_source().map(|s| (tag, s)))
      .for_each(|(tag, asset)| {
        let sri_value = create_digest_from_asset(hash_func, asset);
        add_sri(tag, &sri_value);
      });
  }

  let mut tags = vec![];

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
      PathData::default().filename(output_path.as_str()),
    )
    .always_ok();

  if let Some(title) = &self.config.title {
    tags.push(HTMLPluginTag::create_title(title));
  }

  if let Some(base) = &self.config.base {
    if let Some(tag) = HTMLPluginTag::create_base(base) {
      tags.push(tag);
    }
  }

  let favicon = if let Some(favicon) = &self.config.favicon {
    let favicon = PathBuf::from(favicon)
      .file_name()
      .expect("favicon should have file name")
      .to_string_lossy()
      .to_string();

    let favicon_relative_path = PathBuf::from(self.config.get_relative_path(compilation, &favicon));

    let mut favicon_path: PathBuf = PathBuf::from(self.config.get_public_path(
      compilation,
      favicon_relative_path.to_string_lossy().to_string().as_str(),
    ));

    if favicon_path.to_str().unwrap_or_default().is_empty() {
      favicon_path = compilation
        .options
        .output
        .path
        .as_std_path()
        .join(favicon_relative_path)
        .relative(PathBuf::from(fake_html_file_name).join(".."));
    } else {
      favicon_path.push(favicon_relative_path);
    }

    let mut favicon_link_path = favicon_path.to_string_lossy().to_string();

    if self.config.hash.unwrap_or_default() {
      if let Some(hash) = compilation.get_hash() {
        favicon_link_path = append_hash(&favicon_link_path, hash);
      }
    }

    tags.push(HTMLPluginTag::create_favicon(&generate_posix_path(
      &favicon_link_path,
    )));

    Some(favicon_link_path)
  } else {
    None
  };

  if let Some(meta) = &self.config.meta {
    tags.extend(HTMLPluginTag::create_meta(meta));
  }

  tags.extend(
    asset_tags
      .into_iter()
      .map(|(tag, _)| tag)
      .collect::<Vec<_>>(),
  );

  let mut render_data = serde_json::json!(&self.config.template_parameters);

  let mut body_tags = vec![];
  let mut head_tags = vec![];
  for tag in &tags {
    if tag.tag_name == "script" {
      if matches!(self.config.script_loading, HtmlScriptLoading::Blocking) {
        body_tags.push(tag);
      } else {
        head_tags.push(tag);
      }
    } else {
      head_tags.push(tag);
    }
  }

  merge_json(
    &mut render_data,
    serde_json::json!({
      "htmlRspackPlugin": {
        "tags": {
          "headTags": head_tags,
          "bodyTags": body_tags,
        },
        "files": {
          "favicon": favicon,
          "js": assets.entry("js".into()).or_default(),
          "css": assets.entry("css".into()).or_default(),
          "publicPath": config.get_public_path(compilation, &self.config.filename),
        },
        "options": &self.config
      },
    }),
  );

  // only support "mode" and some fields of "output"
  merge_json(
    &mut render_data,
    serde_json::json!({
      "rspackConfig": {
        "mode": match compilation.options.mode {
          Mode::Development => "development",
          Mode::Production => "production",
          Mode::None => "none",
        },
        "output": {
          "publicPath": config.get_public_path(compilation, &self.config.filename),
          "crossOriginLoading": match &compilation.options.output.cross_origin_loading {
              CrossOriginLoading::Disable => "false",
              CrossOriginLoading::Enable(value) => value,
          },
        }
      },
    }),
  );

  // process with template parameters
  let mut dj = Dojang::new();
  // align escape | unescape with lodash.template syntax https://lodash.com/docs/4.17.15#template which is html-webpack-plugin's default behavior
  dj.with_options(DojangOptions {
    escape: "-".to_string(),
    unescape: "=".to_string(),
  });

  dj.add_function_1("toHtml".into(), render_tag)
    .expect("failed to add template function `renderTag`");

  dj.add_with_option(url.clone(), content.clone())
    .expect("failed to add template");
  let mut template_result = match dj.render(&url, render_data) {
    Ok(compiled) => compiled,
    Err(err) => {
      error_content.push(err.clone());
      compilation.push_diagnostic(Diagnostic::from(miette::Error::msg(err)));
      String::default()
    }
  };

  let has_doctype = template_result.contains("!DOCTYPE") || template_result.contains("!doctype");
  if !has_doctype {
    template_result = format!("<!DOCTYPE html>{template_result}");
  }

  let ast_with_diagnostic = parser.parse_file(&url, template_result)?;

  let (mut current_ast, diagnostic) = ast_with_diagnostic.split_into_parts();

  if !diagnostic.is_empty() {
    compilation.extend_diagnostics(diagnostic);
  }

  if !matches!(self.config.inject, HtmlInject::False) {
    let mut visitor = AssetWriter::new(&tags);
    current_ast.visit_mut_with(&mut visitor);
  }

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
        .filename(output_path.as_str())
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

pub fn render_tag(op: Operand) -> Operand {
  match op {
    Operand::Value(obj) => match serde_json::from_value::<HTMLPluginTag>(obj) {
      Ok(tag) => Operand::Value(Value::from(html_tag_object_to_string(&tag))),
      Err(_) => Operand::Value(Value::from("")),
    },
    Operand::Array(obj) => Operand::Value(Value::from(
      obj
        .iter()
        .map(|val| match render_tag(val.to_owned()) {
          Operand::Value(val) => val.as_str().unwrap_or_default().to_string(),
          _ => "".to_string(),
        })
        .join(""),
    )),
    _ => Operand::Value(Value::from("")),
  }
}
