use std::borrow::Cow;

use swc_core::{
  common::pass::AstKindPath,
  ecma::{
    ast::*,
    visit::{AstParentKind, VisitMut, VisitMutAstPath, VisitMutWith, VisitMutWithPath},
  },
};

use rspack_core::{
  CodeGeneratableContext, CodeGeneratableJavaScriptVisitorBuilder,
  CodeGeneratableJavaScriptVisitors, Compilation, JsAstPath, Module,
};

/// Collect dependency code generation visitors from dependencies of the module passed in.
///
/// Safety:
/// It's only safe to be used if module exists in module graph, or it will panic.
pub fn collect_dependency_code_generation_visitors(
  module: &dyn Module,
  compilation: &Compilation,
) -> (
  CodeGeneratableJavaScriptVisitors,
  CodeGeneratableJavaScriptVisitors,
) {
  let dependencies = compilation
    .module_graph
    .module_graph_module_by_identifier(&module.identifier())
    .map(|mgm| &mgm.dependencies)
    .expect("Failed to get module graph module");

  let mut root_visitors = vec![];
  let mut visitors = vec![];

  let context = CodeGeneratableContext {
    compilation,
    module
  };

  dependencies
    .iter()
    .map(|dependency| dependency.generate(&context)
    .for_each(|code_gen| {
      let js_visitors = code_gen.into_javascript();
      js_visitors.into_iter().for_each(|(ast_path, builder)| {
        if ast_path.is_empty() {
          root_visitors.push((ast_path, builder))
        } else {
          visitors.push((ast_path, builder))
        }
      });
    });

  (root_visitors, visitors)
}

// Invariant: Each [AstPath] in `visitors` contains a value at position `index`.
pub struct ApplyVisitors<'a, 'b> {
  /// `VisitMut` should be shallow. In other words, it should not visit
  /// children of the node.
  visitors: Cow<'b, [(&'a JsAstPath, &'a CodeGeneratableJavaScriptVisitorBuilder)]>,

  index: usize,
}

/// Do two binary searches to find the sub-slice that has `path[index] == kind`.
/// Returns None if no item matches that. `visitors` need to be sorted by path.
fn find_range<'a, 'b>(
  visitors: &'b [(&'a JsAstPath, &'a CodeGeneratableJavaScriptVisitorBuilder)],
  kind: &AstParentKind,
  index: usize,
) -> Option<&'b [(&'a JsAstPath, &'a CodeGeneratableJavaScriptVisitorBuilder)]> {
  // Precondition: visitors is never empty
  let start = if visitors.first().unwrap().0[index] >= *kind {
    // Fast path: It's likely that the whole range is selected
    0
  } else {
    visitors.partition_point(|(path, _)| path[index] < *kind)
  };
  if start >= visitors.len() {
    return None;
  }
  let end = if visitors.last().unwrap().0[index] == *kind {
    // Fast path: It's likely that the whole range is selected
    visitors.len()
  } else {
    visitors[start..].partition_point(|(path, _)| path[index] == *kind) + start
  };
  if end == start {
    return None;
  }
  // Postcondition: return value is never empty
  Some(&visitors[start..end])
}

impl<'a, 'b> ApplyVisitors<'a, 'b> {
  /// `visitors` must have an non-empty [AstPath].
  pub fn new(
    mut visitors: Vec<(&'a JsAstPath, &'a CodeGeneratableJavaScriptVisitorBuilder)>,
  ) -> Self {
    assert!(!visitors.is_empty());
    visitors.sort_by_key(|(path, _)| *path);
    Self {
      visitors: Cow::Owned(visitors),
      index: 0,
    }
  }

  #[inline(never)]
  fn visit_if_required<N>(&mut self, n: &mut N, ast_path: &mut AstKindPath<AstParentKind>)
  where
    N: for<'aa> VisitMutWith<dyn VisitMut + Send + Sync + 'aa>
      + for<'aa, 'bb> VisitMutWithPath<ApplyVisitors<'aa, 'bb>>,
  {
    let mut index = self.index;
    let mut current_visitors = self.visitors.as_ref();
    while index < ast_path.len() {
      let current = index == ast_path.len() - 1;
      let kind = ast_path[index];
      if let Some(visitors) = find_range(current_visitors, &kind, index) {
        // visitors contains all items that match kind at index. Some of them terminate
        // here, some need furth visiting. The terminating items are at the start due to
        // sorting of the list.
        index += 1;

        // skip items that terminate here
        let nested_visitors_start = visitors.partition_point(|(path, _)| path.len() == index);
        if current {
          // Potentially skip visiting this sub tree
          if nested_visitors_start < visitors.len() {
            n.visit_mut_children_with_path(
              &mut ApplyVisitors {
                // We only select visitors starting from `nested_visitors_start`
                // which maintains the invariant.
                visitors: Cow::Borrowed(&visitors[nested_visitors_start..]),
                index,
              },
              ast_path,
            );
          }
          for (_, visitor) in visitors[..nested_visitors_start].iter() {
            n.visit_mut_with(&mut visitor());
          }
          return;
        } else {
          current_visitors = &visitors[nested_visitors_start..];
        }
      } else {
        // Skip visiting this sub tree
        return;
      }
    }
    // Ast path is unchanged, just keep visiting
    n.visit_mut_children_with_path(self, ast_path);
  }
}

macro_rules! method {
  ($name:ident, $T:ty) => {
    fn $name(&mut self, n: &mut $T, ast_path: &mut AstKindPath<AstParentKind>) {
      self.visit_if_required(n, ast_path);
    }
  };
}

impl VisitMutAstPath for ApplyVisitors<'_, '_> {
  // TODO: we need a macro to apply that for all methods
  method!(visit_mut_prop, Prop);
  method!(visit_mut_expr, Expr);
  method!(visit_mut_pat, Pat);
  method!(visit_mut_stmt, Stmt);
  method!(visit_mut_module_decl, ModuleDecl);
  method!(visit_mut_module_item, ModuleItem);
  method!(visit_mut_call_expr, CallExpr);
  method!(visit_mut_lit, Lit);
  method!(visit_mut_str, Str);
}
