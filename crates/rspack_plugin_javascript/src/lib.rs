#![feature(box_patterns)]

mod runtime;
pub mod utils;
pub mod visitors;
use once_cell::sync::Lazy;

pub use runtime::*;

pub mod module;

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
