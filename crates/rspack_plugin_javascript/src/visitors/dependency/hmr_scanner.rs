use rspack_core::ModuleDependency;
use swc_core::common::pass::AstNodePath;
use swc_core::ecma::ast::{CallExpr, Expr, Lit, Str};
use swc_core::ecma::visit::{AstParentNodeRef, VisitAstPath, VisitWithPath};

use super::{
  as_parent_path, is_import_meta_hot_accept_call, is_import_meta_hot_decline_call,
  is_module_hot_accept_call, is_module_hot_decline_call,
};
use crate::dependency::{
  ImportMetaModuleHotAcceptDependency, ImportMetaModuleHotDeclineDependency,
  ModuleHotAcceptDependency, ModuleHotDeclineDependency,
};

pub struct HmrDependencyScanner<'a> {
  pub dependencies: &'a mut Vec<Box<dyn ModuleDependency>>,
  pub flag: (bool, bool, bool, bool),
}

impl<'a> HmrDependencyScanner<'a> {
  pub fn new(dependencies: &'a mut Vec<Box<dyn ModuleDependency>>) -> Self {
    Self {
      dependencies,
      flag: Default::default(),
    }
  }

  fn add_dependency(&mut self, dependency: Box<dyn ModuleDependency>) {
    self.dependencies.push(dependency);
  }
}

impl VisitAstPath for HmrDependencyScanner<'_> {
  fn visit_str<'ast: 'r, 'r>(
    &mut self,
    node: &'ast Str,
    ast_path: &mut AstNodePath<AstParentNodeRef<'r>>,
  ) {
    if self.flag.0 {
      self.add_dependency(box ModuleHotAcceptDependency::new(
        node.value.clone(),
        Some(node.span.into()),
        as_parent_path(ast_path),
      ));
    } else if self.flag.1 {
      self.add_dependency(box ModuleHotDeclineDependency::new(
        node.value.clone(),
        Some(node.span.into()),
        as_parent_path(ast_path),
      ));
    } else if self.flag.2 {
      self.add_dependency(box ImportMetaModuleHotAcceptDependency::new(
        node.value.clone(),
        Some(node.span.into()),
        as_parent_path(ast_path),
      ));
    } else if self.flag.3 {
      self.add_dependency(box ImportMetaModuleHotDeclineDependency::new(
        node.value.clone(),
        Some(node.span.into()),
        as_parent_path(ast_path),
      ));
    }
  }

  fn visit_call_expr<'ast: 'r, 'r>(
    &mut self,
    node: &'ast CallExpr,
    ast_path: &mut AstNodePath<AstParentNodeRef<'r>>,
  ) {
    // avoid nested function call if already enter module.hot.x call
    if self.flag.0 || self.flag.1 || self.flag.2 || self.flag.3 {
      return;
    }

    macro_rules! visit_node_children {
      () => {
        if let Some(first_arg) = node.args.get(0) {
          match first_arg.expr.as_ref() {
            Expr::Lit(Lit::Str(_)) => {
              node.visit_children_with_path(self, ast_path);
            }
            Expr::Array(_) => {
              node.visit_children_with_path(self, ast_path);
            }
            _ => {}
          }
        }
      };
    }

    if is_module_hot_accept_call(node) {
      self.flag.0 = true;
      visit_node_children!();
      self.flag.0 = false;
    } else if is_module_hot_decline_call(node) {
      self.flag.1 = true;
      visit_node_children!();
      self.flag.1 = false;
    } else if is_import_meta_hot_accept_call(node) {
      self.flag.2 = true;
      visit_node_children!();
      self.flag.2 = false;
    } else if is_import_meta_hot_decline_call(node) {
      self.flag.3 = true;
      visit_node_children!();
      self.flag.3 = false;
    } else {
      node.visit_children_with_path(self, ast_path);
    }
  }
}
