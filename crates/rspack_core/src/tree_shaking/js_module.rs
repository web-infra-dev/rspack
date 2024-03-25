use rspack_ast::javascript::Ast;
use swc_core::common::SyntaxContext;

use super::{
  analyzer::OptimizeAnalyzer,
  visitor::{ModuleRefAnalyze, OptimizeAnalyzeResult, SyntaxContextInfo},
};
use crate::needs_refactor::WorkerSyntaxList;
use crate::{BoxDependency, CompilerOptions, ModuleIdentifier};

pub struct JsModule<'b, 'ast: 'b> {
  ast: &'ast Ast,
  worker_syntax_list: &'ast WorkerSyntaxList,
  dependencies: &'b Vec<BoxDependency>,
  module_identifier: ModuleIdentifier,
  compiler_options: &'ast CompilerOptions,
}

impl<'ast, 'b> JsModule<'b, 'ast> {
  pub fn new(
    ast: &'ast Ast,
    worker_syntax_list: &'ast WorkerSyntaxList,
    dependencies: &'b Vec<BoxDependency>,
    module_identifier: ModuleIdentifier,
    compiler_options: &'ast CompilerOptions,
  ) -> Self {
    Self {
      ast,
      worker_syntax_list,
      dependencies,
      module_identifier,
      compiler_options,
    }
  }
}

impl<'a, 'b> OptimizeAnalyzer for JsModule<'a, 'b> {
  fn analyze(&self) -> OptimizeAnalyzeResult {
    self.ast.visit(|program, context| {
      let top_level_mark = context.top_level_mark;
      let unresolved_mark = context.unresolved_mark;
      let unresolved_ctxt = SyntaxContext::empty().apply_mark(unresolved_mark);
      let top_level_ctxt = SyntaxContext::empty().apply_mark(top_level_mark);
      let mut analyzer = ModuleRefAnalyze::new(
        SyntaxContextInfo::new(top_level_ctxt, unresolved_ctxt),
        self.module_identifier,
        self.dependencies,
        self.compiler_options,
        program.comments.as_ref(),
        self.worker_syntax_list,
      );
      program.visit_with(&mut analyzer);
      analyzer.into()
    })
  }
}
