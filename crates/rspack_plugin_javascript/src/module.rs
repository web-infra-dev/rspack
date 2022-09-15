use rspack_error::Result;
use swc::config::{JsMinifyFormatOptions, JsMinifyOptions};
use tracing::instrument;

use crate::visitors::DependencyScanner;
use rspack_core::{
  Compilation, Module, ModuleGraphModule, ModuleRenderResult, ModuleType, SourceType,
};

use std::fmt::Debug;
use swc_common::{FileName, Mark};
use swc_ecma_ast::EsVersion;
use swc_ecma_transforms::{pass::noop, react};
use swc_ecma_visit::VisitMutWith;

use crate::{
  utils::{get_swc_compiler, syntax_by_module_type},
  visitors::finalize,
  HELPERS, JS_HELPERS,
};

pub(crate) static JS_MODULE_SOURCE_TYPE_LIST: &[SourceType; 1] = &[SourceType::JavaScript];
pub struct JsModule {
  pub uri: String,
  pub module_type: ModuleType,
  pub ast: swc_ecma_ast::Program,
  pub source_type_list: &'static [SourceType; 1],
  pub unresolved_mark: Mark,
}

impl Debug for JsModule {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("JsModule")
      .field("uri", &self.uri)
      .field("module_type", &self.module_type)
      .field("ast", &"{..}")
      .finish()
  }
}

impl Module for JsModule {
  #[inline(always)]
  fn module_type(&self) -> ModuleType {
    self.module_type
  }

  #[inline(always)]
  fn source_types(&self) -> &[SourceType] {
    self.source_type_list.as_ref()
  }

  #[instrument]
  fn render(
    &self,
    requested_source_type: SourceType,
    module: &ModuleGraphModule,
    compilation: &Compilation,
  ) -> Result<Option<ModuleRenderResult>> {
    use swc::config::{self as swc_config, SourceMapsConfig};

    if requested_source_type != SourceType::JavaScript {
      return Ok(None);
    }

    let compiler = get_swc_compiler();
    let output = compiler.run(|| {
      HELPERS.set(&JS_HELPERS, || {
        swc::try_with_handler(compiler.cm.clone(), Default::default(), |handler| {
          let fm = compiler
            .cm
            .new_source_file(FileName::Custom(self.uri.to_string()), self.uri.to_string());

          let source_map = false;
          let config = swc_config::Options {
            config: swc_config::Config {
              jsc: swc_config::JscConfig {
                minify: Some(JsMinifyOptions {
                  format: JsMinifyFormatOptions {
                    preserve_annotations: true,
                    ..Default::default()
                  },
                  ..Default::default()
                }),
                target: Some(EsVersion::Es2022),
                syntax: Some(syntax_by_module_type(self.uri.as_str(), &self.module_type)),
                transform: Some(swc_config::TransformConfig {
                  react: react::Options {
                    runtime: Some(react::Runtime::Automatic),
                    ..Default::default()
                  },
                  ..Default::default()
                })
                .into(),
                preserve_all_comments: true.into(),
                ..Default::default()
              },
              inline_sources_content: true.into(),
              // emit_source_map_columns: (!matches!(options.mode, BundleMode::Dev)).into(),
              source_maps: Some(SourceMapsConfig::Bool(source_map)),
              ..Default::default()
            },
            // top_level_mark: Some(bundle_ctx.top_level_mark),
            ..Default::default()
          };
          compiler.process_js_with_custom_pass(
            fm,
            // TODO: It should have a better way rather than clone.
            Some(self.ast.clone()),
            handler,
            &config,
            |_, _| noop(),
            |program, _| {
              // noop()
              dbg!(&program);
              finalize(module, compilation, self.unresolved_mark)
            },
          )
        })
        .unwrap()
      })
    });

    Ok(Some(ModuleRenderResult::JavaScript(output.code)))
  }

  fn dependencies(&mut self) -> Vec<rspack_core::ModuleDependency> {
    let mut dep_scanner = DependencyScanner::default();
    self.ast.visit_mut_with(&mut dep_scanner);
    dep_scanner.dependencies.into_iter().collect()
  }
}
