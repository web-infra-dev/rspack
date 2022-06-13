#![feature(box_patterns)]

mod runtime;
pub mod utils;
pub mod visitors;
use once_cell::sync::Lazy;
use rayon::prelude::*;
pub use runtime::*;
use tracing::instrument;
pub mod module;

use std::fmt::Debug;

use crate::{
  utils::{get_swc_compiler, syntax_by_source_type},
  visitors::finalize,
};
use module::JsModule;
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

static JS_HELPERS: Lazy<Helpers> = Lazy::new(Helpers::default);

#[derive(Debug)]
pub struct JsPlugin {}

impl Plugin for JsPlugin {
  fn register_parse_module(&self, _ctx: PluginContext) -> Option<Vec<rspack_core::SourceType>> {
    Some(vec![
      SourceType::Js,
      SourceType::Jsx,
      SourceType::Ts,
      SourceType::Tsx,
    ])
  }

  #[instrument]
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

  #[instrument(skip_all)]
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
    let ordered_modules = chunk.ordered_modules(module_graph);
    let code = ordered_modules
      .par_iter()
      .filter(|module| {
        matches!(
          module.source_type,
          SourceType::Js | SourceType::Ts | SourceType::Tsx | SourceType::Jsx
        )
      })
      .map(|module| module.module.render(module, compilation))
      .chain([{
        if chunk.kind.is_entry() {
          format!(
            "rs.require(\"{}\")",
            ordered_modules
              .last()
              .ok_or_else(|| anyhow::format_err!("TODO:"))?
              .id
              .as_str()
          )
        } else {
          String::new()
        }
      }])
      .fold(String::new, |mut output, cur| {
        output += &cur;
        output
      })
      .collect();
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
