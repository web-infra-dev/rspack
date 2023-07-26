use super::{
  analyzer::OptimizeAnalyzer,
  visitor::{MarkInfo, ModuleRefAnalyze, OptimizeAnalyzeResult},
};
use crate::{
  ast::javascript::Ast, Dependency, ModuleDependency, ModuleGraphModule, ModuleIdentifier,
};

pub struct JsModule<'b, 'a: 'b> {
  ast: &'a Ast,
  dependencies: &'b Vec<Box<dyn ModuleDependency>>,
  module_identifier: ModuleIdentifier,
}

impl<'a, 'b> JsModule<'b, 'a> {
  pub fn new(
    ast: &'a Ast,
    dependencies: &'b Vec<Box<dyn ModuleDependency>>,
    module_identifier: ModuleIdentifier,
  ) -> Self {
    Self {
      ast,
      dependencies,
      module_identifier,
    }
  }
}

impl<'a, 'b> OptimizeAnalyzer for JsModule<'a, 'b> {
  fn analyze(&self, compilation: &crate::Compilation) -> OptimizeAnalyzeResult {
    self.ast.visit(|program, context| {
      let top_level_mark = context.top_level_mark;
      let unresolved_mark = context.unresolved_mark;
      let mut worker_syntax_scanner = crate::needs_refactor::WorkerSyntaxScanner::new(
        crate::needs_refactor::DEFAULT_WORKER_SYNTAX,
      );
      program.visit_with(&mut worker_syntax_scanner);
      let worker_syntax_list = &worker_syntax_scanner.into();

      let mut analyzer = ModuleRefAnalyze::new(
        MarkInfo::new(top_level_mark, unresolved_mark),
        self.module_identifier,
        &self.dependencies,
        &compilation.options,
        program.comments.as_ref(),
        worker_syntax_list,
      );
      program.visit_with(&mut analyzer);
      analyzer.into()
    })
  }
}
