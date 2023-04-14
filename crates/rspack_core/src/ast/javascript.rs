use std::{hash::Hash, sync::Arc};

use anyhow::Error;
use swc_core::base::{try_with_handler, SwcComments};
use swc_core::common::pass::{AstKindPath, AstNodePath};
use swc_core::common::{
  errors::Handler, sync::Lrc, util::take::Take, Globals, Mark, SourceMap, GLOBALS,
};
use swc_core::ecma::ast::{Module, Program as SwcProgram};
use swc_core::ecma::transforms::base::helpers;
use swc_core::ecma::transforms::base::helpers::Helpers;
use swc_core::ecma::visit::{
  AstParentKind, AstParentNodeRef, Fold, FoldWith, Visit, VisitAll, VisitAllWith, VisitAstPath,
  VisitMut, VisitMutAstPath, VisitMutWith, VisitMutWithPath, VisitWith, VisitWithPath,
};

/// Program is a wrapper for SwcProgram
///
/// Use this to avoid using `use swc_ecma_visit::*`
/// and save changes in self
#[derive(Clone)]
pub struct Program {
  pub(crate) program: SwcProgram,
  pub comments: Option<SwcComments>,
}

impl std::hash::Hash for Program {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.program.hash(state);
  }
}

impl std::fmt::Debug for Program {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Program")
      .field("program", &self.program)
      .field("comments", &"...")
      .finish()
  }
}

impl Program {
  pub fn new(program: SwcProgram, comments: Option<SwcComments>) -> Self {
    Self { program, comments }
  }

  pub fn fold_with<V: ?Sized + Fold>(&mut self, v: &mut V) {
    let p = std::mem::replace(&mut self.program, SwcProgram::Module(Module::dummy()));
    self.program = p.fold_with(v);
  }

  pub fn visit_with<V: ?Sized + Visit>(&self, v: &mut V) {
    self.program.visit_with(v)
  }

  pub fn visit_mut_with<V: ?Sized + VisitMut>(&mut self, v: &mut V) {
    self.program.visit_mut_with(v)
  }

  pub fn visit_with_path<'ast, 'r, V: ?Sized + VisitAstPath>(
    &'ast self,
    v: &mut V,
    ast_path: &mut AstNodePath<AstParentNodeRef<'r>>,
  ) where
    'ast: 'r,
  {
    self.program.visit_with_path(v, ast_path)
  }

  pub fn visit_mut_with_path<V: ?Sized + VisitMutAstPath>(
    &mut self,
    v: &mut V,
    ast_path: &mut AstKindPath<AstParentKind>,
  ) {
    self.program.visit_mut_with_path(v, ast_path)
  }

  pub fn visit_all_with<V: ?Sized + VisitAll>(&self, v: &mut V) {
    self.program.visit_all_with(v)
  }

  pub fn get_inner_program(&self) -> &SwcProgram {
    &self.program
  }
}

/// Swc transform context
pub struct Context {
  pub globals: Globals,
  pub helpers: Helpers,
  pub top_level_mark: Mark,
  pub unresolved_mark: Mark,
  //  comments: swcComments,
  pub source_map: Arc<SourceMap>,
}

impl Context {
  pub fn new(source_map: Arc<SourceMap>) -> Self {
    let globals: Globals = Default::default();
    // generate preset mark & helpers
    let (top_level_mark, unresolved_mark, helpers) =
      GLOBALS.set(&globals, || (Mark::new(), Mark::new(), Helpers::new(true)));

    Self {
      globals,
      helpers,
      top_level_mark,
      unresolved_mark,
      source_map,
    }
  }
}

impl std::fmt::Debug for Context {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Context")
      .field("helpers", &self.helpers)
      .field("top_level_mark", &self.top_level_mark)
      .field("unresolved_mark", &self.unresolved_mark)
      .finish()
  }
}

/// The global javascript ast
#[derive(Debug, Clone)]
pub struct Ast {
  program: Program,
  context: Arc<Context>,
}

impl Hash for Ast {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.program.hash(state);
    // TODO: De we need to implement Hash for `context`?
    // self.context.hash(state);
  }
}

impl Ast {
  pub fn new(
    program: SwcProgram,
    source_map: Arc<SourceMap>,
    comments: Option<SwcComments>,
  ) -> Self {
    Self {
      program: Program::new(program, comments),
      context: Arc::new(Context::new(source_map)),
    }
  }

  pub fn get_context(&self) -> &Context {
    &self.context
  }

  pub fn transform<F, R>(&mut self, f: F) -> R
  where
    F: FnOnce(&mut Program, &Context) -> R,
  {
    let Self { program, context } = self;
    GLOBALS.set(&context.globals, || {
      helpers::HELPERS.set(&context.helpers, || f(program, context))
    })
  }

  pub fn transform_with_handler<F, R>(&mut self, cm: Lrc<SourceMap>, f: F) -> Result<R, Error>
  where
    F: FnOnce(&Handler, &mut Program, &Context) -> Result<R, Error>,
  {
    self.transform(|program, context| {
      try_with_handler(cm, Default::default(), |handler| {
        f(handler, program, context)
      })
    })
  }

  pub fn visit<F, R>(&self, f: F) -> R
  where
    F: FnOnce(&Program, &Context) -> R,
  {
    let Self { program, context } = self;
    GLOBALS.set(&context.globals, || {
      helpers::HELPERS.set(&context.helpers, || f(program, context))
    })
  }
}
