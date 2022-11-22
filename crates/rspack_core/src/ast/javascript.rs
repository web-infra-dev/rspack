use anyhow::Error;
use std::{hash::Hash, sync::Arc};
use swc_common::{errors::Handler, sync::Lrc, util::take::Take, Globals, Mark, SourceMap, GLOBALS};
use swc_ecma_ast::{Module, Program as SwcProgram};
use swc_ecma_transforms::helpers::Helpers;
use swc_ecma_visit::{Fold, FoldWith, Visit, VisitAll, VisitAllWith, VisitWith};

/// Program is a wrapper for SwcProgram
///
/// Use this to avoid using `use swc_ecma_visit::*`
/// and save changes in self
#[derive(Debug, Clone, Hash)]
pub struct Program(SwcProgram);

impl Program {
  pub fn fold_with<V: ?Sized + Fold>(&mut self, v: &mut V) {
    let p = std::mem::replace(&mut self.0, SwcProgram::Module(Module::dummy()));
    self.0 = p.fold_with(v);
  }

  pub fn visit_with<V: ?Sized + Visit>(&self, v: &mut V) {
    self.0.visit_with(v)
  }

  pub fn visit_all_with<V: ?Sized + VisitAll>(&self, v: &mut V) {
    self.0.visit_all_with(v)
  }

  pub fn get_inner_program(&self) -> &SwcProgram {
    &self.0
  }
}

/// Swc transform context
pub struct Context {
  pub globals: Globals,
  pub helpers: Helpers,
  pub top_level_mark: Mark,
  pub unresolved_mark: Mark,
  //  comments: swcComments,
  //  source_map: swcSourceMap
}

impl Context {
  pub fn new() -> Self {
    let globals: Globals = Default::default();
    // generate preset mark & helpers
    let (top_level_mark, unresolved_mark, helpers) =
      GLOBALS.set(&globals, || (Mark::new(), Mark::new(), Helpers::new(true)));

    Self {
      globals,
      helpers,
      top_level_mark,
      unresolved_mark,
    }
  }
}

impl Default for Context {
  fn default() -> Self {
    Context::new()
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
  pub fn new(program: SwcProgram) -> Self {
    Self {
      program: Program(program),
      context: Arc::new(Context::new()),
    }
  }

  pub fn transform<F, R>(&mut self, f: F) -> R
  where
    F: FnOnce(&mut Program, &Context) -> R,
  {
    let Self { program, context } = self;
    GLOBALS.set(&context.globals, || {
      swc_ecma_transforms::helpers::HELPERS.set(&context.helpers, || f(program, context))
    })
  }

  pub fn transform_with_handler<F, R>(&mut self, cm: Lrc<SourceMap>, f: F) -> Result<R, Error>
  where
    F: FnOnce(&Handler, &mut Program, &Context) -> Result<R, Error>,
  {
    self.transform(|program, context| {
      swc::try_with_handler(cm, Default::default(), |handler| {
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
      swc_ecma_transforms::helpers::HELPERS.set(&context.helpers, || f(program, context))
    })
  }
}
