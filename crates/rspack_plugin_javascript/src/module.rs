use rspack_error::Result;
use tracing::instrument;

use crate::visitors::DependencyScanner;
use rspack_core::{
  Compilation, Module, ModuleGraphModule, ModuleRenderResult, ModuleType, SourceType,
};

use std::fmt::Debug;
use swc_common::{FileName, Mark};
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
          compiler.process_js_with_custom_pass(
            fm,
            // TODO: It should have a better way rather than clone.
            Some(self.ast.clone()),
            handler,
            &swc_config::Options {
              config: swc_config::Config {
                jsc: swc_config::JscConfig {
                  target: compilation.options.target.es_version,
                  syntax: Some(syntax_by_module_type(self.uri.as_str(), &self.module_type)),
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
                minify: swc::BoolConfig::new(Some(compilation.options.builtins.minify)),
                env: if compilation.options.target.platform.is_browsers_list() {
                  Some(swc_ecma_preset_env::Config {
                    mode: if compilation.options.builtins.polyfill {
                      Some(swc_ecma_preset_env::Mode::Usage)
                    } else {
                      Some(swc_ecma_preset_env::Mode::Entry)
                    },
                    targets: Some(swc_ecma_preset_env::Targets::Query(
                      preset_env_base::query::Query::Multiple(
                        compilation.options.builtins.browserslist.clone(),
                      ),
                    )),
                    ..Default::default()
                  })
                } else {
                  None
                },
                ..Default::default()
              },
              // top_level_mark: Some(bundle_ctx.top_level_mark),
              ..Default::default()
            },
            |_, _| noop(),
            |_, _| {
              // noop()
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
