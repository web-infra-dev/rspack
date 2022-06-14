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

mod plugin;
pub use plugin::*;

static JS_HELPERS: Lazy<Helpers> = Lazy::new(Helpers::default);

// use typemap::{Key, TypeMap};

// pub struct JsAst;

// #[derive(Debug)]
// pub struct Value(swc_ecma_ast::Program);

// impl Key for JsAst {
//   type Value = Value;
// }

// fn transofrm(mut ctx: TypeMap) {
//   // let mut map = TypeMap::new();
//   ctx.insert::<JsAst>(Value(swc_ecma_ast::Program::Module(
//     swc_ecma_ast::Module::dummy(),
//   )));
// }
