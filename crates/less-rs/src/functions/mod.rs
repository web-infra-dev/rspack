//! Built-in functions (`functions/*.js`) and the function registry.

mod builtins;
mod color_fns;
pub(crate) mod io;

use std::sync::Arc;

use cow_utils::CowUtils;
use rustc_hash::FxHashMap;

use crate::{error::LResult, eval::EvalCtx, plugin::LessFunction, tree::*};

/// A built-in function. Receives the evaluation context, the call position
/// and the (already filtered) arguments; `Ok(None)` means "return undefined"
/// (the call is then output as plain CSS).
pub(crate) type Builtin =
  for<'c, 'a, 's> fn(&'c mut EvalCtx<'a, 's>, Pos, &[Node<'a>]) -> LResult<Option<Node<'a>>>;

#[derive(Clone)]
pub(crate) enum FnEntry {
  Builtin { f: Builtin, eval_args: bool },
  Custom(Arc<dyn LessFunction>),
}

/// The function registry (built-ins plus functions registered by the host).
#[derive(Clone)]
pub struct Registry {
  map: FxHashMap<String, FnEntry>,
  pub(crate) pre: Vec<Arc<dyn crate::plugin::PreProcessor>>,
  pub(crate) post: Vec<Arc<dyn crate::plugin::PostProcessor>>,
}

impl std::fmt::Debug for Registry {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Registry")
      .field("functions", &self.map.len())
      .finish()
  }
}

impl Default for Registry {
  fn default() -> Registry {
    Registry::with_builtins()
  }
}

impl Registry {
  pub fn with_builtins() -> Registry {
    let mut r = Registry {
      map: FxHashMap::default(),
      pre: Vec::new(),
      post: Vec::new(),
    };
    builtins::register(&mut r);
    color_fns::register(&mut r);
    io::register(&mut r);
    r
  }

  pub(crate) fn add(&mut self, name: &str, f: Builtin) {
    self.map.insert(
      name.cow_to_ascii_lowercase().into_owned(),
      FnEntry::Builtin { f, eval_args: true },
    );
  }
  pub(crate) fn add_raw(&mut self, name: &str, f: Builtin) {
    self.map.insert(
      name.cow_to_ascii_lowercase().into_owned(),
      FnEntry::Builtin {
        f,
        eval_args: false,
      },
    );
  }

  /// Registers a host-provided function (overrides a built-in of the same name).
  pub fn add_function(&mut self, name: &str, f: Arc<dyn LessFunction>) {
    self.map.insert(
      name.cow_to_ascii_lowercase().into_owned(),
      FnEntry::Custom(f),
    );
  }

  pub fn add_pre_processor(&mut self, p: Arc<dyn crate::plugin::PreProcessor>) {
    self.pre.push(p);
  }
  pub fn add_post_processor(&mut self, p: Arc<dyn crate::plugin::PostProcessor>) {
    self.post.push(p);
  }

  pub(crate) fn preprocess(&self, src: &str, filename: &str) -> Option<String> {
    if self.pre.is_empty() {
      return None;
    }
    let mut s = src.to_string();
    for p in &self.pre {
      s = p.process(&s, filename);
    }
    Some(s)
  }

  pub(crate) fn get(&self, lower_name: &str) -> Option<FnEntry> {
    self.map.get(lower_name).cloned()
  }
}

/// `FunctionCaller.call(args)`
pub(crate) fn call<'a>(
  ctx: &mut EvalCtx<'a, '_>,
  entry: FnEntry,
  _name: &str,
  c: &'a Call<'a>,
  args: &'a [Node<'a>],
) -> LResult<Option<Node<'a>>> {
  let eval_args = match &entry {
    FnEntry::Builtin { eval_args, .. } => *eval_args,
    FnEntry::Custom(f) => f.eval_args(),
  };
  let mut list: smallvec::SmallVec<[Node<'a>; 8]> = smallvec::SmallVec::with_capacity(args.len());
  for a in args {
    let item = if eval_args { ctx.eval(*a)? } else { *a };
    if item.is_comment() {
      continue;
    }
    let item = match item {
      Node::Expression(e) => {
        let sub: smallvec::SmallVec<[Node<'a>; 8]> = e
          .value
          .iter()
          .copied()
          .filter(|n| !n.is_comment())
          .collect();
        if sub.len() == 1 {
          if e.parens && matches!(sub[0], Node::Operation(o) if o.op == "/") {
            item
          } else {
            sub[0]
          }
        } else if sub.len() == e.value.len() {
          // no comments removed: an identical expression
          Node::Expression(ctx.a.expression_owned(e.value, None))
        } else {
          Node::Expression(ctx.a.expression(&sub, None))
        }
      }
      other => other,
    };
    list.push(item);
  }
  match entry {
    FnEntry::Builtin { f, .. } => f(ctx, c.pos, &list),
    FnEntry::Custom(f) => crate::plugin::call_custom(ctx, c.pos, &*f, &list),
  }
}
