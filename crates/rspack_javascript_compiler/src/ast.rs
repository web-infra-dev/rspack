use std::{hash::Hash, sync::Arc};

use swc_core::{
  common::{GLOBALS, Globals, Mark, SourceMap, comments::SingleThreadedComments, util::take::Take},
  ecma::{
    ast::{Module, Program as SwcProgram},
    visit::{Visit, VisitWith},
  },
};

/// Program is a wrapper for SwcProgram
///
/// Use this to avoid using `use swc_ecma_visit::*`
/// and save changes in self
#[derive(Clone)]
pub struct Program {
  pub(crate) program: SwcProgram,
  pub comments: Option<SingleThreadedComments>,
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
  pub fn new(program: SwcProgram, comments: Option<SingleThreadedComments>) -> Self {
    Self { program, comments }
  }

  pub fn visit_with<V: ?Sized + Visit>(&self, v: &mut V) {
    self.program.visit_with(v)
  }
}

impl Take for Program {
  fn dummy() -> Self {
    Self {
      program: SwcProgram::Module(Module::dummy()),
      comments: None,
    }
  }
}

/// Swc transform context
pub struct Context {
  pub globals: Globals,
  pub top_level_mark: Mark,
  pub unresolved_mark: Mark,
  pub source_map: Arc<SourceMap>,
}

impl Context {
  pub fn new(source_map: Arc<SourceMap>, globals: Option<Globals>) -> Self {
    let globals = globals.unwrap_or_default();
    // generate preset mark & helpers
    let (top_level_mark, unresolved_mark) = GLOBALS.set(&globals, || (Mark::new(), Mark::new()));

    Self {
      globals,
      top_level_mark,
      unresolved_mark,
      source_map,
    }
  }
}

impl std::fmt::Debug for Context {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Context")
      .field("top_level_mark", &self.top_level_mark)
      .field("unresolved_mark", &self.unresolved_mark)
      .finish()
  }
}

impl Take for Context {
  fn dummy() -> Self {
    Self::new(Arc::new(SourceMap::new(Default::default())), None)
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

impl Default for Ast {
  fn default() -> Self {
    Self {
      program: Program::dummy(),
      context: Arc::new(Context::dummy()),
    }
  }
}

impl Ast {
  pub fn new(
    program: SwcProgram,
    source_map: Arc<SourceMap>,
    comments: Option<SingleThreadedComments>,
  ) -> Self {
    Self {
      program: Program::new(program, comments),
      context: Arc::new(Context::new(source_map, None)),
    }
  }

  pub fn with_context(mut self, context: Context) -> Self {
    self.context = Arc::new(context);
    self
  }

  pub fn visit<F, R>(&self, f: F) -> R
  where
    F: FnOnce(&Program, &Context) -> R,
  {
    let Self { program, context } = self;
    GLOBALS.set(&context.globals, || f(program, context))
  }
}
