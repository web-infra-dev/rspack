pub mod ast_sugar;
pub mod name_helpers;
pub mod side_effect;
use once_cell::sync::Lazy;
use std::{path::Path, sync::Arc};
use swc::{config::IsModule, Compiler};
use tracing::instrument;

use swc_ecma_ast::{ModuleDecl, ModuleItem};

use swc_common::{FileName, FilePathMapping, SourceMap};
use swc_ecma_parser::Syntax;
use swc_ecma_parser::{EsConfig, TsConfig};

mod statement;
pub use statement::*;
pub mod log;

#[inline]
pub fn is_external_module(source: &str) -> bool {
  source.starts_with("node:")
}

#[inline]
pub fn is_decl_or_stmt(node: &ModuleItem) -> bool {
  matches!(
    node,
    ModuleItem::ModuleDecl(
      ModuleDecl::ExportDecl(_)
        | ModuleDecl::ExportDefaultExpr(_)
        | ModuleDecl::ExportDefaultDecl(_)
    ) | ModuleItem::Stmt(_)
  )
}

static COMPILER: Lazy<Arc<Compiler>> = Lazy::new(|| {
  let cm = Arc::new(SourceMap::new(FilePathMapping::empty()));

  Arc::new(Compiler::new(cm))
});

pub use rspack_core::get_swc_compiler;
