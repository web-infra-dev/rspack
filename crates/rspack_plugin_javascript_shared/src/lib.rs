//! How to add a missing hook to `JsAstVisitorHook`
//! Find the missing hook at https://rustdoc.swc.rs/swc_ecma_visit/trait.Visit.html
mod js_ast_visitor;

pub use js_ast_visitor::{JsAstVisitor, JsAstVisitorHook};
