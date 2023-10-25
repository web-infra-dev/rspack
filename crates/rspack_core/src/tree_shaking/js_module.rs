use rspack_ast::javascript::Ast;
use swc_core::common::SyntaxContext;

use super::{
  analyzer::OptimizeAnalyzer,
  visitor::{ModuleRefAnalyze, OptimizeAnalyzeResult, SyntaxContextInfo},
};
use crate::{BoxDependency, CompilerOptions, ModuleIdentifier};

pub struct JsModule<'b, 'a: 'b> {
  ast: &'a Ast,
  dependencies: &'b Vec<BoxDependency>,
  module_identifier: ModuleIdentifier,
  compiler_options: &'a CompilerOptions,
}

impl<'a, 'b> JsModule<'b, 'a> {
  pub fn new(
    ast: &'a Ast,
    dependencies: &'b Vec<BoxDependency>,
    module_identifier: ModuleIdentifier,
    compiler_options: &'a CompilerOptions,
  ) -> Self {
    Self {
      ast,
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
      let mut worker_syntax_scanner = crate::needs_refactor::WorkerSyntaxScanner::new(
        crate::needs_refactor::DEFAULT_WORKER_SYNTAX,
      );
      program.visit_with(&mut worker_syntax_scanner);
      let worker_syntax_list = &worker_syntax_scanner.into();

      let mut analyzer = ModuleRefAnalyze::new(
        SyntaxContextInfo::new(top_level_ctxt, unresolved_ctxt),
        self.module_identifier,
        self.dependencies,
        self.compiler_options,
        program.comments.as_ref(),
        worker_syntax_list,
      );
      program.visit_with(&mut analyzer);
      analyzer.into()
    })
  }
}
