#![feature(let_chains)]
#![feature(box_patterns)]
#![feature(box_syntax)]
#![allow(dead_code)]

// use once_cell::sync::Lazy;

pub(crate) mod dependency;
mod plugin;
pub use plugin::*;
mod ast;
pub mod runtime;
pub mod utils;
pub mod visitors;

// static JS_HELPERS: Lazy<Helpers> = Lazy::new(Helpers::default);

// use typemap::{Key, TypeMap};

// pub struct JsAst;

// #[derive(Debug)]
// pub struct Value(swc_ecma_ast::Program);

// impl Key for JsAst {
//   type Value = Value;
// }

// fn transform(mut ctx: TypeMap) {
//   // let mut map = TypeMap::new();
//   ctx.insert::<JsAst>(Value(swc_ecma_ast::Program::Module(
//     swc_ecma_ast::Module::dummy(),
//   )));
// }
