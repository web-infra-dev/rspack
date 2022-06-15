use tracing::instrument;

use crate::visitors::DependencyScanner;
use rspack_core::{Compilation, Module, ModuleGraphModule, SourceType};
use std::fmt::Debug;
use swc_common::FileName;
use swc_ecma_ast::EsVersion;
use swc_ecma_transforms::{pass::noop, react};
use swc_ecma_visit::VisitMutWith;

use crate::{
  utils::{get_swc_compiler, syntax_by_source_type},
  visitors::finalize,
  HELPERS, JS_HELPERS,
};

pub struct JsModule {
  pub uri: String,
  pub source_type: SourceType,
  pub ast: swc_ecma_ast::Program,
}

impl Debug for JsModule {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("JsModule")
      .field("uri", &self.uri)
      .field("source_type", &self.source_type)
      .field("ast", &"{..}")
      .finish()
  }
}

impl Module for JsModule {
  #[instrument]
  fn render(&self, module: &ModuleGraphModule, compilation: &Compilation) -> String {
    use swc::config::{self as swc_config, SourceMapsConfig};
    let compiler = get_swc_compiler();
    let output = compiler.run(|| {
      HELPERS.set(&JS_HELPERS, || {
        swc::try_with_handler(compiler.cm.clone(), Default::default(), |handler| {
          let fm = compiler
            .cm
            .new_source_file(FileName::Custom(self.uri.to_string()), self.uri.to_string());

          let source_map = false;
          compiler.process_js_with_custom_pass(
            fm,
            // TODO: It should have a better way rather than clone.
            Some(self.ast.clone()),
            handler,
            &swc_config::Options {
              config: swc_config::Config {
                jsc: swc_config::JscConfig {
                  target: Some(EsVersion::Es2022),
                  syntax: Some(syntax_by_source_type(self.uri.as_str(), &self.source_type)),
                  transform: Some(swc_config::TransformConfig {
                    react: react::Options {
                      runtime: Some(react::Runtime::Automatic),
                      ..Default::default()
                    },
                    ..Default::default()
                  })
                  .into(),
                  ..Default::default()
                },
                inline_sources_content: true.into(),
                // emit_source_map_columns: (!matches!(options.mode, BundleMode::Dev)).into(),
                source_maps: Some(SourceMapsConfig::Bool(source_map)),
                ..Default::default()
              },
              // top_level_mark: Some(bundle_ctx.top_level_mark),
              ..Default::default()
            },
            |_, _| noop(),
            |_, _| {
              // noop()
              finalize(module, compilation)
            },
          )
        })
        .unwrap()
      })
    });
    output.code
  }

  fn dependencies(&mut self) -> Vec<rspack_core::ModuleDependency> {
    let mut dep_scanner = DependencyScanner::default();
    self.ast.visit_mut_with(&mut dep_scanner);
    dep_scanner.dependencies.into_iter().collect()
  }
}
