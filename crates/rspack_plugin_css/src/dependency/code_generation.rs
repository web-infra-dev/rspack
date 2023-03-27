/**
 * Some code is modified based on
 * https://github.com/vercel/turbo/blob/a1947f64443fb98e5c3e10bca6ef9eafd278bd21/crates/turbopack-css/src/path_visitor.rs
 * MPL-2.0 Licensed
 * Author ForsakenHarmony
 * Copyright (c)
 * https://github.com/vercel/turbo/blob/a1947f64443fb98e5c3e10bca6ef9eafd278bd21/LICENSE#L1
 */
use std::borrow::Cow;

use rspack_core::{
  CodeGeneratableContext, CodeGeneratableCssResult, CodeGeneratableCssVisitors,
  CodeGeneratableDeclMappings, CssAstPath, CssVisitorBuilder, GenerateContext, Module,
};
use rspack_error::Result;
use rustc_hash::FxHashMap as HashMap;
use swc_core::{
  common::pass::AstKindPath,
  css::{
    ast::Url,
    visit::{AstParentKind, VisitMut, VisitMutAstPath, VisitMutWith, VisitMutWithPath},
  },
};

pub struct DependencyCodeGenerationVisitors {
  pub visitors: CodeGeneratableCssVisitors,
  pub root_visitors: CodeGeneratableCssVisitors,
  pub decl_mappings: CodeGeneratableDeclMappings,
}

pub fn collect_dependency_code_generation_visitors(
  module: &dyn Module,
  generate_context: &mut GenerateContext,
) -> Result<DependencyCodeGenerationVisitors> {
  let compilation = generate_context.compilation;

  let dependencies = compilation
    .module_graph
    .module_graph_module_by_identifier(&module.identifier())
    .map(|mgm| &mgm.dependencies)
    .expect("Failed to get module graph module");

  let mut root_visitors = vec![];
  let mut visitors = vec![];

  let mut context = CodeGeneratableContext {
    compilation,
    module,
    runtime_requirements: generate_context.runtime_requirements,
  };

  let mut mappings = HashMap::default();

  let mut code_generatable_result = dependencies
    .iter()
    .map(|id| {
      compilation
        .module_graph
        .dependency_by_id(id)
        .expect("should have dependency")
        .generate(&mut context)
    })
    .collect::<Result<Vec<_>>>()?;

  if let Some(dependencies) = module.get_presentational_dependencies() {
    code_generatable_result.extend(
      dependencies
        .iter()
        .map(|dependency| dependency.generate(&mut context))
        .collect::<Result<Vec<_>>>()?,
    );
  }

  code_generatable_result.into_iter().for_each(|code_gen| {
    let CodeGeneratableCssResult {
      visitors: raw_visitors,
      decl_mappings,
    } = code_gen.into_css();
    mappings.extend(decl_mappings);
    raw_visitors.into_iter().for_each(|(ast_path, builder)| {
      if ast_path.is_empty() {
        root_visitors.push((ast_path, builder))
      } else {
        visitors.push((ast_path, builder))
      }
    });
  });

  Ok(DependencyCodeGenerationVisitors {
    visitors,
    root_visitors,
    decl_mappings: mappings,
  })
}

pub struct DependencyVisitor<'a, 'b> {
  visitors: Cow<'b, [(&'a CssAstPath, &'a dyn CssVisitorBuilder)]>,
  last_index: usize,
}

impl<'a, 'b> DependencyVisitor<'a, 'b> {
  pub(crate) fn new(mut visitors: Vec<(&'a CssAstPath, &'a dyn CssVisitorBuilder)>) -> Self {
    debug_assert!(!visitors.is_empty(), "There should be at least one visitor");
    // We should sort the visitor in JsAstPath's lexical order, or the partition will be wrong.
    visitors.sort_unstable_by_key(|(ast_path, _)| *ast_path);

    Self {
      visitors: Cow::Owned(visitors),
      last_index: 0,
    }
  }

  /// Filter the visitors with the range of the given index targeting to the given path.
  ///
  /// Prerequisite: Visitors should be sorted or there's no guarantee of the result.
  ///
  /// The macro generates the visitor and the enum `AstParentKind` ensures
  /// the order of visiting and the order of the enum `AstParentKind`.
  /// So comparing the `AstParentKind` is guaranteed to be correct.
  ///
  /// Example:
  ///
  /// If `AstParentKind::A` is greater than `AstParentKind::B`, which means,
  /// under the same parent, `AstParentKind::B` is visited before `AstParentKind::A`.
  fn filter_visitors_by_indexed_path(
    visitors: &'b [(&'a CssAstPath, &'a dyn CssVisitorBuilder)],
    index: usize,
    path: Option<&AstParentKind>,
  ) -> &'b [(&'a CssAstPath, &'a dyn CssVisitorBuilder)] {
    // Since the visitor is sorted, if the first ast path for the first visitor is greater than the given path,
    // then there's no visitor that matches the given path. So we can return an empty slice.

    if visitors
      .first()
      .expect("There should be at least one visitor")
      .0
      .get(index)
      > path
    {
      return &[];
    }

    // The same applies to the last ast path for the last visitor.
    if visitors
      .last()
      .expect("There should be at least one visitor")
      .0
      .get(index)
      < path
    {
      return &[];
    }

    // Otherwise, we can partition the visitors by the given path and find the starting point of the visitors.
    let starting_point = if visitors
      .first()
      .expect("There should be at least one visitor")
      .0
      .get(index)
      == path
    {
      0
    } else {
      visitors.partition_point(|(ast_path, _)| ast_path.get(index) < path)
    };

    if starting_point >= visitors.len() {
      return &[];
    }

    // If the path for the partition point is still greater than the given path, then there's no chance they will be equal.
    if visitors
      .get(starting_point)
      .and_then(|(ast_path, _)| ast_path.get(index))
      > path
    {
      return &[];
    }

    // Find the pivot where the path is greater than the given path. If there's no more path that is greater than the given path,
    // the ending point will be the length of the visitors.
    let ending_point = visitors[starting_point..]
      .partition_point(|(ast_path, _)| ast_path.get(index) == path)
      + starting_point;

    &visitors[starting_point..ending_point]
  }

  fn visit_with_condition<N>(&mut self, node: &mut N, ast_path: &mut AstKindPath<AstParentKind>)
  where
    N: for<'aa, 'bb> VisitMutWithPath<DependencyVisitor<'aa, 'bb>>
      + for<'aa> VisitMutWith<dyn VisitMut + 'aa>,
  {
    let mut index = self.last_index;
    let mut current_visitors = &*self.visitors;

    while index < ast_path.len() {
      // We have some ast paths left to do, so that we should test if these ast paths match the given ast path for each index.
      let path_at_index = ast_path.get(index);
      let visitors =
        DependencyVisitor::filter_visitors_by_indexed_path(current_visitors, index, path_at_index);

      if visitors.is_empty() {
        // If there's no visitors for the path at the given index,
        // then we will never find the matching path later, so there's nothing left we can do.
        return;
      }

      // Check if there are some visitors' ast paths end here.
      let ending_point = visitors.partition_point(|(ast_path, _)| index == ast_path.len() - 1);
      if index == ast_path.len() - 1 {
        // If we reach here, we have all ast paths matched.

        // There's more ast paths left, so we should continue to visit the children.
        if ending_point < visitors.len() {
          node.visit_mut_children_with_path(
            &mut DependencyVisitor {
              visitors: Cow::Borrowed(&visitors[ending_point..]),
              last_index: ast_path.len() - 1,
            },
            ast_path,
          );
        }

        // We finally found these visitors that end here, so we can apply the visitors to it.
        visitors[..ending_point].iter().for_each(|(_, visitor)| {
          node.visit_mut_with(&mut visitor.create());
        });
        return;
      } else {
        // Otherwise, we can advance to the next path at the given index to check the possibility.
        index += 1;
        current_visitors = &visitors[ending_point..];
      }
    }

    // For those who don't have any ast path left to ensure, we can visit the children directly.
    node.visit_mut_children_with_path(self, ast_path);
  }
}

macro_rules! impl_ast_node_interceptor {
  ($ident:ident, $node:ident) => {
    ::paste::paste! {
      fn [<visit_mut_ $ident>](
        &mut self,
        node: &mut $node,
        ast_path: &mut AstKindPath<AstParentKind>,
      ) {
        self.visit_with_condition(node, ast_path);
      }
    }
  };
}

impl<'a, 'b> VisitMutAstPath for DependencyVisitor<'a, 'b> {
  impl_ast_node_interceptor!(url, Url);
}
