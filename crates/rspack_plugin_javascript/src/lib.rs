#![feature(box_patterns)]

mod runtime;
pub mod utils;
pub mod visitors;
use once_cell::sync::Lazy;
pub use runtime::*;

use std::fmt::Debug;

use rspack_core::{
  Asset, AssetFilename, BoxModule, Compilation, JobContext, Module, ModuleGraphModule,
  ParseModuleArgs, Plugin, PluginContext, PluginRenderManifestHookOutput, SourceType,
};
use swc_common::{util::take::Take, FileName};
use swc_ecma_ast::EsVersion;
use swc_ecma_transforms::{pass::noop, react};
use swc_ecma_visit::VisitMutWith;
use utils::parse_file;
use visitors::DependencyScanner;

use crate::{
  utils::{get_swc_compiler, syntax_by_source_type},
  visitors::finalize,
};

static JS_HELPERS: Lazy<Helpers> = Lazy::new(Helpers::default);

#[derive(Debug)]
pub struct JsPlugin {}

struct JsModule {
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

impl Plugin for JsPlugin {
  fn register_parse_module(&self, _ctx: PluginContext) -> Option<Vec<rspack_core::SourceType>> {
    Some(vec![
      SourceType::Js,
      SourceType::Jsx,
      SourceType::Ts,
      SourceType::Tsx,
    ])
  }

  fn parse_module(&self, ctx: PluginContext<&mut JobContext>, args: ParseModuleArgs) -> BoxModule {
    let ast = parse_file(
      args.source,
      args.uri,
      ctx.context.source_type.as_ref().unwrap(),
    );
    Box::new(JsModule {
      ast,
      uri: args.uri.to_string(),
      source_type: ctx.context.source_type.unwrap(),
    })
  }

  fn render_manifest(
    &self,
    _ctx: PluginContext,
    args: rspack_core::RenderManifestArgs,
  ) -> PluginRenderManifestHookOutput {
    let compilation = args.compilation;
    let module_graph = &compilation.module_graph;
    let chunk = compilation
      .chunk_graph
      .chunk_by_id(args.chunk_id)
      .ok_or_else(|| anyhow::format_err!("Not found chunk {:?}", args.chunk_id))?;

    let ordered_js_modules = chunk
      .ordered_modules(module_graph)
      .into_iter()
      .filter(|module| {
        matches!(
          module.source_type,
          SourceType::Js | SourceType::Ts | SourceType::Tsx | SourceType::Jsx
        )
      });
    let code = ordered_js_modules
      .into_iter()
      .map(|module| module.module.render(module, compilation))
      .fold(String::new(), |mut output, cur| {
        output += &cur;
        output
      });
    Ok(vec![Asset {
      rendered: code,
      filename: AssetFilename::Static(format!("{}.js", args.chunk_id)),
    }])
  }
}

use typemap::{Key, TypeMap};

pub struct JsAst;

#[derive(Debug)]
pub struct Value(swc_ecma_ast::Program);

impl Key for JsAst {
  type Value = Value;
}

fn transofrm(mut ctx: TypeMap) {
  // let mut map = TypeMap::new();
  ctx.insert::<JsAst>(Value(swc_ecma_ast::Program::Module(
    swc_ecma_ast::Module::dummy(),
  )));
}
