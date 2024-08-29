use std::{
  collections::{hash_map::DefaultHasher, HashMap},
  fs,
  hash::{Hash, Hasher},
  path::{Path, PathBuf},
  sync::LazyLock,
};

use anyhow::Context;
use itertools::Itertools;
use rayon::prelude::*;
use rspack_core::{
  parse_to_url,
  rspack_sources::{RawSource, SourceExt},
  Compilation, CompilationAsset, CompilationId, CompilationProcessAssets, CrossOriginLoading,
  FilenameTemplate, Mode, PathData, Plugin,
};
use rspack_dojang::dojang::{Dojang, DojangOptions};
use rspack_dojang::Operand;
use rspack_error::{miette, AnyhowError, Diagnostic, Result};
use rspack_hook::{plugin, plugin_hook};
use rspack_paths::AssertUtf8;
use rspack_util::{
  fx_hash::{BuildFxHasher, FxDashMap},
  infallible::ResultInfallibleExt as _,
};
use serde_json::Value;
use sugar_path::SugarPath;
use swc_html::visit::VisitMutWith;

use crate::{
  config::{HtmlInject, HtmlRspackPluginOptions, HtmlScriptLoading, TemplateParameters},
  parser::HtmlCompiler,
  sri::{add_sri, create_digest_from_asset},
  visitors::{
    asset::AssetWriter,
    tag::HtmlPluginTag,
    utils::{append_hash, generate_posix_path, html_tag_object_to_string, merge_json},
  },
  AfterEmitData, AfterTemplateExecutionData, AlterAssetTagGroupsData, AlterAssetTagsData,
  BeforeAssetTagGenerationData, BeforeEmitData, HtmlPluginAssetTags, HtmlPluginAssets,
  HtmlPluginHooks,
};

pub enum Renderer {
  Template(String),
  Function,
}

static COMPILATION_HOOKS_MAP: LazyLock<FxDashMap<CompilationId, Box<HtmlPluginHooks>>> =
  LazyLock::new(Default::default);

#[plugin]
#[derive(Debug)]
pub struct HtmlRspackPlugin {
  config: HtmlRspackPluginOptions,
}

struct HtmlTemplateData {
  pub render: Renderer,
  pub url: String,
  pub filename: String,
  pub error: Option<AnyhowError>,
  pub file_dependencies: Vec<PathBuf>,
}

impl HtmlRspackPlugin {
  pub fn new(config: HtmlRspackPluginOptions) -> Self {
    Self::new_inner(config)
  }

  pub fn get_compilation_hooks(
    id: CompilationId,
  ) -> dashmap::mapref::one::Ref<'static, CompilationId, Box<HtmlPluginHooks>, BuildFxHasher> {
    if !COMPILATION_HOOKS_MAP.contains_key(&id) {
      COMPILATION_HOOKS_MAP.insert(id, Default::default());
    }
    COMPILATION_HOOKS_MAP
      .get(&id)
      .expect("should have js plugin drive")
  }

  pub fn get_compilation_hooks_mut(
    compilation: &Compilation,
  ) -> dashmap::mapref::one::RefMut<'_, CompilationId, Box<HtmlPluginHooks>, BuildFxHasher> {
    COMPILATION_HOOKS_MAP.entry(compilation.id()).or_default()
  }

  fn get_template_data(&self, compilation: &Compilation) -> HtmlTemplateData {
    if let Some(content) = &self.config.template_content {
      HtmlTemplateData {
        render: if self.config.template_fn.is_some() {
          Renderer::Function
        } else {
          Renderer::Template(content.clone())
        },
        url: parse_to_url("template_content.html").path().to_string(),
        filename: "template_content.html".to_string(),
        error: None,
        file_dependencies: vec![],
      }
    } else if let Some(template) = &self.config.template {
      // TODO: support loader query form
      let resolved_template = path_clean::clean(
        compilation
          .options
          .context
          .as_path()
          .join(template.as_str()),
      )
      .assert_utf8();
      let url = resolved_template.as_str().to_string();

      if self.config.template_fn.is_some() {
        HtmlTemplateData {
          render: Renderer::Function,
          url,
          filename: template.clone(),
          error: None,
          file_dependencies: vec![],
        }
      } else {
        let content = fs::read_to_string(&resolved_template)
          .context(format!(
            "HtmlRspackPlugin: could not load file `{}` from `{}`",
            template, &compilation.options.context
          ))
          .map_err(AnyhowError::from);

        match content {
          Ok(content) => HtmlTemplateData {
            render: Renderer::Template(content),
            url,
            filename: template.clone(),
            error: None,
            file_dependencies: vec![resolved_template.into_std_path_buf()],
          },
          Err(err) => HtmlTemplateData {
            render: Renderer::Template(default_template().to_owned()),
            url: parse_to_url("default.html").path().to_string(),
            filename: template.clone(),
            error: Some(err),
            file_dependencies: vec![],
          },
        }
      }
    } else {
      let default_src_template =
        path_clean::clean(compilation.options.context.as_path().join("src/index.ejs"))
          .assert_utf8();

      if let Ok(content) = fs::read_to_string(&default_src_template) {
        HtmlTemplateData {
          render: Renderer::Template(content),
          url: default_src_template.as_str().to_string(),
          filename: "src/index.ejs".to_string(),
          error: None,
          file_dependencies: vec![default_src_template.into_std_path_buf()],
        }
      } else {
        HtmlTemplateData {
          render: Renderer::Template(default_template().to_owned()),
          url: parse_to_url("default.html").path().to_string(),
          filename: "default.html".to_string(),
          error: None,
          file_dependencies: vec![],
        }
      }
    }
  }

  fn get_assets_info<'a>(
    &self,
    compilation: &'a Compilation,
    public_path: &str,
    html_file_name: &str,
  ) -> (HtmlPluginAssets, HashMap<String, &'a CompilationAsset>) {
    let mut assets = HtmlPluginAssets::default();
    let mut asset_map = HashMap::new();
    assets.public_path = public_path.to_string();

    let included_assets = compilation
      .entrypoints
      .keys()
      .filter(|&entry_name| {
        let mut included = true;
        if let Some(included_chunks) = &self.config.chunks {
          included = included_chunks.iter().any(|c| c.eq(entry_name));
        }
        if let Some(exclude_chunks) = &self.config.exclude_chunks {
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

    for (asset_name, asset) in included_assets {
      if let Some(extension) =
        Path::new(asset_name.split("?").next().unwrap_or_default()).extension()
      {
        let mut asset_uri = format!("{}{}", assets.public_path, url_encode_path(&asset_name));
        if self.config.hash.unwrap_or_default() {
          if let Some(hash) = compilation.get_hash() {
            asset_uri = append_hash(&asset_uri, hash);
          }
        }
        let final_path = generate_posix_path(&asset_uri);
        if extension.eq_ignore_ascii_case("css") {
          assets.css.push(final_path.to_string());
          asset_map.insert(final_path.to_string(), asset);
        } else if extension.eq_ignore_ascii_case("js") || extension.eq_ignore_ascii_case("mjs") {
          assets.js.push(final_path.to_string());
          asset_map.insert(final_path.to_string(), asset);
        }
      }
    }

    assets.favicon = if let Some(favicon) = &self.config.favicon {
      let favicon = PathBuf::from(favicon)
        .file_name()
        .expect("favicon should have file name")
        .to_string_lossy()
        .to_string();

      let favicon_relative_path =
        PathBuf::from(self.config.get_relative_path(compilation, &favicon));

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
          .relative(PathBuf::from(html_file_name).join(".."));
      } else {
        favicon_path.push(favicon_relative_path);
      }

      let mut favicon_link_path = favicon_path.to_string_lossy().to_string();

      if self.config.hash.unwrap_or_default() {
        if let Some(hash) = compilation.get_hash() {
          favicon_link_path = append_hash(&favicon_link_path, hash);
        }
      }

      Some(generate_posix_path(&favicon_link_path).into())
    } else {
      None
    };

    (assets, asset_map)
  }
}

#[plugin_hook(CompilationProcessAssets for HtmlRspackPlugin, stage = Compilation::PROCESS_ASSETS_STAGE_OPTIMIZE_INLINE)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  let config = &self.config;
  let hooks = HtmlRspackPlugin::get_compilation_hooks(compilation.id());
  let public_path = self
    .config
    .get_public_path(compilation, &self.config.filename);

  let mut error_content = vec![];

  let parser = HtmlCompiler::new(config);

  let template = self.get_template_data(compilation);

  for dep in template.file_dependencies {
    compilation.file_dependencies.insert(dep);
  }

  if let Some(err) = template.error {
    error_content.push(err.to_string());
    compilation.push_diagnostic(Diagnostic::from(miette::Error::from(err)));
  }

  // Use the same filename as template
  let output_path = compilation.options.output.path.join(template.filename);

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

  let assets_info = self.get_assets_info(compilation, &public_path, &fake_html_file_name);

  let before_generation_data = hooks
    .before_asset_tag_generation
    .call(BeforeAssetTagGenerationData {
      assets: assets_info.0,
      // TODO: support named html
      output_name: String::new(),
    })
    .await?;

  let mut asset_tags = HtmlPluginAssetTags::default();

  // create script tags
  for script in &before_generation_data.assets.js {
    asset_tags
      .scripts
      .push(HtmlPluginTag::create_script(script, &config.script_loading));
  }

  // create style tags
  for style in &before_generation_data.assets.css {
    asset_tags.styles.push(HtmlPluginTag::create_style(style));
  }

  // create base tag
  if let Some(base) = &self.config.base {
    if let Some(tag) = HtmlPluginTag::create_base(base) {
      asset_tags.meta.push(tag);
    }
  }

  // create title tag
  if let Some(title) = &self.config.title {
    asset_tags.meta.push(HtmlPluginTag::create_title(title));
  }

  // create meta tags
  if let Some(meta) = &self.config.meta {
    asset_tags.meta.extend(HtmlPluginTag::create_meta(meta));
  }

  // create favicon tag
  if let Some(favicon) = &before_generation_data.assets.favicon {
    asset_tags.meta.push(HtmlPluginTag::create_favicon(favicon));
  }

  // if some plugin changes assets in the same stage after this plugin
  // both the name and the integrity may be inaccurate
  if let Some(hash_func) = &config.sri {
    asset_tags
      .scripts
      .par_iter_mut()
      .filter_map(|tag| {
        if let Some(asset) = tag
          .asset
          .as_ref()
          .and_then(|asset| assets_info.1.get(asset))
        {
          asset.get_source().map(|s| (tag, s))
        } else {
          None
        }
      })
      .for_each(|(tag, asset)| {
        let sri_value = create_digest_from_asset(hash_func, asset);
        add_sri(tag, &sri_value);
      });
    asset_tags
      .styles
      .par_iter_mut()
      .filter_map(|tag| {
        if let Some(asset) = tag
          .asset
          .as_ref()
          .and_then(|asset| assets_info.1.get(asset))
        {
          asset.get_source().map(|s| (tag, s))
        } else {
          None
        }
      })
      .for_each(|(tag, asset)| {
        let sri_value = create_digest_from_asset(hash_func, asset);
        add_sri(tag, &sri_value);
      });
  }

  let alter_asset_tags_data = hooks
    .alter_asset_tags
    .call(AlterAssetTagsData {
      asset_tags,
      public_path: public_path.clone(),
      // TODO: support named html
      output_name: String::new(),
    })
    .await?;

  let mut body_tags = vec![];
  let mut head_tags = vec![];

  head_tags.extend(alter_asset_tags_data.asset_tags.meta);
  head_tags.extend(alter_asset_tags_data.asset_tags.styles);

  for tag in &alter_asset_tags_data.asset_tags.scripts {
    match self.config.inject {
      HtmlInject::Head => head_tags.push(tag.to_owned()),
      HtmlInject::Body => body_tags.push(tag.to_owned()),
      HtmlInject::False => {
        if matches!(self.config.script_loading, HtmlScriptLoading::Blocking) {
          body_tags.push(tag.to_owned());
        } else {
          head_tags.push(tag.to_owned());
        }
      }
    }
  }

  let alter_asset_tag_groups_data = hooks
    .alter_asset_tag_groups
    .call(AlterAssetTagGroupsData {
      head_tags,
      body_tags,
      public_path: public_path.clone(),
      output_name: String::new(),
    })
    .await?;

  let parameters = if matches!(
    self.config.template_parameters,
    TemplateParameters::Disabled
  ) {
    serde_json::json!({})
  } else {
    let mut res = serde_json::json!({});

    merge_json(
      &mut res,
      serde_json::json!({
        "htmlRspackPlugin": {
          "tags": {
            "headTags": &alter_asset_tag_groups_data.head_tags,
            "bodyTags": &alter_asset_tag_groups_data.body_tags,
          },
          "files": &before_generation_data.assets,
          "options": &self.config,
        },
      }),
    );

    // only support "mode" and some fields of "output"
    merge_json(
      &mut res,
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

    match &self.config.template_parameters {
      TemplateParameters::Map(data) => {
        merge_json(&mut res, serde_json::json!(&data));
      }
      TemplateParameters::Function(func) => {
        let func_res = (func.inner)(
          serde_json::to_string(&res).unwrap_or_else(|_| panic!("invalid json to_string")),
        )
        .await;
        match func_res {
          Ok(new_data) => match serde_json::from_str(&new_data) {
            Ok(data) => res = data,
            Err(err) => {
              error_content.push(format!(
                "HtmlRspackPlugin: failed to parse template parameters: {err}",
              ));
              compilation.push_diagnostic(Diagnostic::from(miette::Error::msg(err)));
            }
          },
          Err(err) => {
            error_content.push(format!(
              "HtmlRspackPlugin: failed to generate template parameters: {err}",
            ));
            compilation.push_diagnostic(Diagnostic::from(miette::Error::msg(err)));
          }
        }
      }
      TemplateParameters::Disabled => {}
    };

    res
  };

  let template_execution_result = match template.render {
    Renderer::Template(content) => {
      // process with template parameters
      let mut dj = Dojang::new();
      // align escape | unescape with lodash.template syntax https://lodash.com/docs/4.17.15#template which is html-webpack-plugin's default behavior
      dj.with_options(DojangOptions {
        escape: "-".to_string(),
        unescape: "=".to_string(),
      });

      dj.add_function_1("toHtml".into(), render_tag)
        .expect("failed to add template function `renderTag`");

      dj.add_with_option(template.url.clone(), content.clone())
        .expect("failed to add template");

      match dj.render(&template.url, parameters) {
        Ok(compiled) => compiled,
        Err(err) => {
          error_content.push(err.clone());
          compilation.push_diagnostic(Diagnostic::from(miette::Error::msg(err)));
          String::default()
        }
      }
    }
    Renderer::Function => {
      let res = (config
        .template_fn
        .as_ref()
        .unwrap_or_else(|| unreachable!())
        .inner)(
        serde_json::to_string(&parameters).unwrap_or_else(|_| panic!("invalid json to_string")),
      )
      .await;

      match res {
        Ok(compiled) => compiled,
        Err(err) => {
          error_content.push(err.to_string());
          compilation.push_diagnostic(Diagnostic::from(miette::Error::msg(err)));
          String::default()
        }
      }
    }
  };

  let mut after_template_execution_data = hooks
    .after_template_execution
    .call(AfterTemplateExecutionData {
      html: template_execution_result,
      head_tags: alter_asset_tag_groups_data.head_tags.clone(),
      body_tags: alter_asset_tag_groups_data.body_tags.clone(),
      output_name: String::new(),
    })
    .await?;

  let has_doctype = after_template_execution_data.html.contains("!DOCTYPE")
    || after_template_execution_data.html.contains("!doctype");
  if !has_doctype {
    after_template_execution_data.html =
      format!("<!DOCTYPE html>{}", after_template_execution_data.html);
  }

  let ast_with_diagnostic = parser.parse_file(&template.url, after_template_execution_data.html)?;

  let (mut current_ast, diagnostic) = ast_with_diagnostic.split_into_parts();

  if !diagnostic.is_empty() {
    compilation.extend_diagnostics(diagnostic);
  }

  if !matches!(self.config.inject, HtmlInject::False) {
    let mut visitor = AssetWriter::new(
      &alter_asset_tag_groups_data.head_tags,
      &alter_asset_tag_groups_data.body_tags,
    );
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

  let before_emit_data = hooks
    .before_emit
    .call(BeforeEmitData {
      html: source,
      output_name: String::new(),
    })
    .await?;

  let hash = hash_for_source(&before_emit_data.html);

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
    CompilationAsset::new(
      Some(RawSource::from(before_emit_data.html).boxed()),
      asset_info,
    ),
  );

  let _ = hooks
    .after_emit
    .call(AfterEmitData {
      output_name: String::new(),
    })
    .await?;

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
    Operand::Value(obj) => match serde_json::from_value::<HtmlPluginTag>(obj) {
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
