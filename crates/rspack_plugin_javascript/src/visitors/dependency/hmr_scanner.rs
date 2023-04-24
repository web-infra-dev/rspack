use bitflags::bitflags;
use rspack_core::ModuleDependency;
use swc_core::common::pass::AstNodePath;
use swc_core::ecma::ast::{CallExpr, Expr, Lit, Str};
use swc_core::ecma::visit::{AstParentNodeRef, VisitAstPath, VisitWithPath};

use super::{as_parent_path, is_module_hot_accept_call, is_module_hot_decline_call};
use crate::dependency::{
  ImportMetaModuleHotAcceptDependency, ImportMetaModuleHotDeclineDependency,
  ModuleHotAcceptDependency, ModuleHotDeclineDependency,
};
use crate::visitors::{is_import_meta_hot_accept_call, is_import_meta_hot_decline_call};

bitflags! {
  #[derive(Default)]
  pub struct HmrScannerFlag: u8 {
    const MODULE_HOT_ACCEPT = 1 << 0;
    const MODULE_HOT_DECLINE = 1 << 1;
    const IMPORT_META_MODULE_HOT_ACCEPT = 1 << 2;
    const IMPORT_META_MODULE_HOT_DECLINE = 1 << 3;
  }
}

pub struct HmrDependencyScanner<'a> {
  pub dependencies: &'a mut Vec<Box<dyn ModuleDependency>>,
  pub flag: HmrScannerFlag,
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
    if self.flag.contains(HmrScannerFlag::MODULE_HOT_ACCEPT) {
      self.add_dependency(box ModuleHotAcceptDependency::new(
        node.value.clone(),
        Some(node.span.into()),
        as_parent_path(ast_path),
      ));
    } else if self.flag.contains(HmrScannerFlag::MODULE_HOT_DECLINE) {
      self.add_dependency(box ModuleHotDeclineDependency::new(
        node.value.clone(),
        Some(node.span.into()),
        as_parent_path(ast_path),
      ));
    } else if self
      .flag
      .contains(HmrScannerFlag::IMPORT_META_MODULE_HOT_ACCEPT)
    {
      self.add_dependency(box ImportMetaModuleHotAcceptDependency::new(
        node.value.clone(),
        Some(node.span.into()),
        as_parent_path(ast_path),
      ));
    } else if self
      .flag
      .contains(HmrScannerFlag::IMPORT_META_MODULE_HOT_DECLINE)
    {
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
    if self
      .flag
      .contains(HmrScannerFlag::IMPORT_META_MODULE_HOT_ACCEPT)
      || self
        .flag
        .contains(HmrScannerFlag::IMPORT_META_MODULE_HOT_DECLINE)
      || self.flag.contains(HmrScannerFlag::MODULE_HOT_ACCEPT)
      || self.flag.contains(HmrScannerFlag::MODULE_HOT_DECLINE)
    {
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
      self.flag.insert(HmrScannerFlag::MODULE_HOT_ACCEPT);
      visit_node_children!();
      self.flag.remove(HmrScannerFlag::MODULE_HOT_ACCEPT);
    } else if is_module_hot_decline_call(node) {
      self.flag.insert(HmrScannerFlag::MODULE_HOT_DECLINE);
      visit_node_children!();
      self.flag.insert(HmrScannerFlag::MODULE_HOT_DECLINE);
    } else if is_import_meta_hot_accept_call(node) {
      self
        .flag
        .insert(HmrScannerFlag::IMPORT_META_MODULE_HOT_ACCEPT);
      visit_node_children!();
      self
        .flag
        .insert(HmrScannerFlag::IMPORT_META_MODULE_HOT_ACCEPT);
    } else if is_import_meta_hot_decline_call(node) {
      self
        .flag
        .insert(HmrScannerFlag::IMPORT_META_MODULE_HOT_DECLINE);
      visit_node_children!();
      self
        .flag
        .insert(HmrScannerFlag::IMPORT_META_MODULE_HOT_DECLINE);
    } else {
      node.visit_children_with_path(self, ast_path);
    }
  }
}
