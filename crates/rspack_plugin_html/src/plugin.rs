use std::{
  borrow::Cow,
  path::{Path, PathBuf},
  sync::LazyLock,
};

use cow_utils::CowUtils;
use rspack_core::{
  Compilation, CompilationId, CompilationProcessAssets, Filename, FilenameTemplate, NoFilenameFn,
  Plugin,
};
use rspack_error::{miette, Diagnostic, Result};
use rspack_hook::{plugin, plugin_hook};
use rspack_util::fx_hash::FxDashMap;
use sugar_path::SugarPath;
use swc_html::visit::VisitMutWith;

use crate::{
  asset::{create_favicon_asset, create_html_asset, HtmlPluginAssetTags, HtmlPluginAssets},
  config::{HtmlInject, HtmlRspackPluginOptions},
  injector::AssetInjector,
  parser::HtmlCompiler,
  template::HtmlTemplate,
  AfterEmitData, AfterTemplateExecutionData, AlterAssetTagGroupsData, AlterAssetTagsData,
  BeforeAssetTagGenerationData, BeforeEmitData, HtmlPluginHooks,
};

static COMPILATION_HOOKS_MAP: LazyLock<FxDashMap<CompilationId, Box<HtmlPluginHooks>>> =
  LazyLock::new(Default::default);

#[plugin]
#[derive(Debug)]
pub struct HtmlRspackPlugin {
  config: HtmlRspackPluginOptions,
}

impl HtmlRspackPlugin {
  pub fn new(config: HtmlRspackPluginOptions) -> Self {
    Self::new_inner(config)
  }

  pub fn get_compilation_hooks(
    id: CompilationId,
  ) -> dashmap::mapref::one::Ref<'static, CompilationId, Box<HtmlPluginHooks>> {
    if !COMPILATION_HOOKS_MAP.contains_key(&id) {
      COMPILATION_HOOKS_MAP.insert(id, Default::default());
    }
    COMPILATION_HOOKS_MAP
      .get(&id)
      .expect("should have js plugin drive")
  }

  pub fn get_compilation_hooks_mut(
    id: CompilationId,
  ) -> dashmap::mapref::one::RefMut<'static, CompilationId, Box<HtmlPluginHooks>> {
    COMPILATION_HOOKS_MAP.entry(id).or_default()
  }
}

async fn generate_html(
  filename: &str,
  html_file_name: &Filename<NoFilenameFn>,
  config: &HtmlRspackPluginOptions,
  compilation: &mut Compilation,
  hooks: &HtmlPluginHooks,
) -> Result<(String, String, Vec<PathBuf>), miette::Error> {
  let public_path = config.get_public_path(compilation, filename);

  let mut template = HtmlTemplate::new(config, compilation)?;

  let template_file_name = compilation
    .options
    .output
    .path
    .join(template.filename.clone());

  let assets_info = HtmlPluginAssets::create_assets(
    config,
    compilation,
    &public_path,
    &template_file_name,
    html_file_name,
  );

  let before_generation_data = hooks
    .before_asset_tag_generation
    .call(BeforeAssetTagGenerationData {
      assets: assets_info.0,
      output_name: html_file_name.as_str().to_string(),
      compilation_id: compilation.id(),
    })
    .await?;

  let asset_tags: HtmlPluginAssetTags =
    HtmlPluginAssetTags::from_assets(config, &before_generation_data.assets, &assets_info.1);

  let alter_asset_tags_data = hooks
    .alter_asset_tags
    .call(AlterAssetTagsData {
      asset_tags,
      public_path: public_path.clone(),
      output_name: html_file_name.as_str().to_string(),
      compilation_id: compilation.id(),
    })
    .await?;

  let (head_tags, body_tags) =
    HtmlPluginAssetTags::to_groups(config, alter_asset_tags_data.asset_tags);

  let alter_asset_tag_groups_data = hooks
    .alter_asset_tag_groups
    .call(AlterAssetTagGroupsData {
      head_tags,
      body_tags,
      public_path: public_path.clone(),
      output_name: html_file_name.as_str().to_string(),
      compilation_id: compilation.id(),
    })
    .await?;

  template
    .create_parameters(
      filename,
      config,
      &alter_asset_tag_groups_data.head_tags,
      &alter_asset_tag_groups_data.body_tags,
      &before_generation_data.assets,
      compilation,
    )
    .await?;

  let template_execution_result = template.render(config).await?;

  let mut after_template_execution_data = hooks
    .after_template_execution
    .call(AfterTemplateExecutionData {
      html: template_execution_result,
      head_tags: alter_asset_tag_groups_data.head_tags,
      body_tags: alter_asset_tag_groups_data.body_tags,
      output_name: html_file_name.as_str().to_string(),
      compilation_id: compilation.id(),
    })
    .await?;

  let has_doctype = after_template_execution_data.html.contains("!DOCTYPE")
    || after_template_execution_data.html.contains("!doctype");
  if !has_doctype {
    after_template_execution_data.html =
      format!("<!DOCTYPE html>{}", after_template_execution_data.html);
  }

  let parser = HtmlCompiler::new(config);

  let ast_with_diagnostic = parser.parse_file(&template.url, after_template_execution_data.html)?;

  let (mut current_ast, diagnostic) = ast_with_diagnostic.split_into_parts();

  if !diagnostic.is_empty() {
    compilation.extend_diagnostics(diagnostic);
  }

  if !matches!(config.inject, HtmlInject::False) {
    let mut visitor = AssetInjector::new(
      &after_template_execution_data.head_tags,
      &after_template_execution_data.body_tags,
    );
    current_ast.visit_mut_with(&mut visitor);
  }

  let raw_html = parser.codegen(&mut current_ast, compilation)?;
  let html = raw_html.cow_replace("$$RSPACK_URL_AMP$$", "&");

  let html = if has_doctype {
    html
  } else {
    html.cow_replace("<!DOCTYPE html>", "")
  };

  Ok((
    template_file_name.to_string(),
    html.into_owned(),
    template.file_dependencies,
  ))
}

#[plugin_hook(CompilationProcessAssets for HtmlRspackPlugin, stage = Compilation::PROCESS_ASSETS_STAGE_OPTIMIZE_INLINE)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  let config: &HtmlRspackPluginOptions = &self.config;
  let hooks = HtmlRspackPlugin::get_compilation_hooks(compilation.id());

  // TODO: parallel generate html
  for filename in &config.filename {
    let filename = filename.cow_replace("[templatehash]", "[contenthash]");

    // convert absolute filename into relative so that webpack can
    // generate it at correct location
    let filename = {
      let filename_path = Path::new(filename.as_ref());
      if filename_path.is_absolute() {
        let output_path = &compilation.options.output.path;
        Cow::from(
          filename_path
            .relative(output_path)
            .to_string_lossy()
            .to_string(),
        )
      } else {
        filename
      }
    };

    let output_file_name = FilenameTemplate::from(filename.to_string());

    let (template_file_name, html) = match generate_html(
      filename.as_ref(),
      &output_file_name,
      config,
      compilation,
      &hooks,
    )
    .await
    {
      Ok(content) => {
        compilation
          .file_dependencies
          .extend(content.2.into_iter().map(Into::into));
        (content.0, content.1)
      }
      Err(err) => {
        let error_msg = err.to_string();
        compilation.push_diagnostic(Diagnostic::from(err));
        ("error.html".to_string(), create_error_html(&error_msg))
      }
    };

    let mut before_emit_data = hooks
      .before_emit
      .call(BeforeEmitData {
        html,
        output_name: output_file_name.as_str().to_string(),
        compilation_id: compilation.id(),
      })
      .await?;

    if let Some(favicon) = &config.favicon {
      match create_favicon_asset(favicon, config, compilation) {
        Ok(favicon) => compilation.emit_asset(favicon.0, favicon.1),
        Err(err) => {
          let error_msg = err.to_string();
          compilation.push_diagnostic(Diagnostic::from(err));
          before_emit_data.html = create_error_html(&error_msg);
        }
      };
    }

    let html_asset = create_html_asset(
      &output_file_name,
      &before_emit_data.html,
      &template_file_name,
      compilation,
    );

    compilation.emit_asset(html_asset.0.clone(), html_asset.1);

    let _ = hooks
      .after_emit
      .call(AfterEmitData {
        output_name: html_asset.0.to_string(),
        compilation_id: compilation.id(),
      })
      .await?;
  }

  Ok(())
}

impl Plugin for HtmlRspackPlugin {
  fn name(&self) -> &'static str {
    "rspack.HtmlRspackPlugin"
  }

  fn apply(
    &self,
    ctx: rspack_core::PluginContext<&mut rspack_core::ApplyContext>,
    _options: &rspack_core::CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .compilation_hooks
      .process_assets
      .tap(process_assets::new(self));
    Ok(())
  }
}

fn create_error_html(err: &str) -> String {
  format!(
    r#"Html Rspack Plugin:
    <pre>
      Error: {err}
    </pre>
    "#,
  )
}
