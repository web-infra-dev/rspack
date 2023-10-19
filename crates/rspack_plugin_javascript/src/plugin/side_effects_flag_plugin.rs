use async_trait::async_trait;
use rspack_core::{Compilation, ConnectionState, ModuleGraph, Plugin, ResolvedExportInfoTarget};
use rspack_error::Result;
use rustc_hash::FxHashSet as HashSet;
// use rspack_core::Plugin;
// use rspack_error::Result;
use swc_core::common::{Span, Spanned, SyntaxContext, GLOBALS};
use swc_core::ecma::ast::*;
use swc_core::ecma::utils::{contains_arguments, ExprCtx, ExprExt};
use swc_core::ecma::visit::{noop_visit_type, Visit, VisitWith};

use crate::dependency::{
  HarmonyExportImportedSpecifierDependency, HarmonyImportSpecifierDependency,
};

#[derive(Debug)]
pub struct SideEffectsFlagPluginVisitor {
  unresolved_ctxt: SyntaxContext,
  pub side_effects_span: Option<Span>,
  is_top_level: bool,
}

#[derive(Debug)]
pub struct SyntaxContextInfo {
  unresolved_ctxt: SyntaxContext,
}

impl SyntaxContextInfo {
  pub fn new(unresolved_ctxt: SyntaxContext) -> Self {
    Self { unresolved_ctxt }
  }
}

impl SideEffectsFlagPluginVisitor {
  pub fn new(mark_info: SyntaxContextInfo) -> Self {
    Self {
      unresolved_ctxt: mark_info.unresolved_ctxt,
      side_effects_span: None,
      is_top_level: true,
    }
  }
}

impl Visit for SideEffectsFlagPluginVisitor {
  noop_visit_type!();
  fn visit_program(&mut self, node: &Program) {
    assert!(GLOBALS.is_set());
    node.visit_children_with(self);
  }

  fn visit_module(&mut self, node: &Module) {
    for module_item in &node.body {
      if !is_import_decl(module_item) {
        module_item.visit_with(self);
      }
    }
  }

  fn visit_script(&mut self, node: &Script) {
    for stmt in &node.body {
      stmt.visit_with(self);
    }
  }

  fn visit_stmt(&mut self, node: &Stmt) {
    if !self.is_top_level {
      return;
    }
    self.analyze_stmt_side_effects(node);
    node.visit_children_with(self);
  }

  fn visit_class_member(&mut self, node: &ClassMember) {
    if let Some(key) = node.class_key() && key.is_computed() {
      key.visit_with(self);
    }

    let top_level = self.is_top_level;
    self.is_top_level = false;
    node.visit_children_with(self);
    self.is_top_level = top_level;
  }

  fn visit_fn_decl(&mut self, node: &FnDecl) {
    let top_level = self.is_top_level;
    self.is_top_level = false;
    node.visit_children_with(self);
    self.is_top_level = top_level;
  }

  fn visit_fn_expr(&mut self, node: &FnExpr) {
    let top_level = self.is_top_level;
    self.is_top_level = false;
    node.visit_children_with(self);
    self.is_top_level = top_level;
  }

  fn visit_arrow_expr(&mut self, node: &ArrowExpr) {
    let top_level = self.is_top_level;
    self.is_top_level = false;
    node.visit_children_with(self);
    self.is_top_level = top_level;
  }
}

impl SideEffectsFlagPluginVisitor {
  /// If we find a stmt that has side effects, we will skip the rest of the stmts.
  /// And mark the module as having side effects.
  fn analyze_stmt_side_effects(&mut self, ele: &Stmt) {
    if self.side_effects_span.is_some() {
      return;
    }
    match ele {
      Stmt::If(stmt) => {
        if !is_pure_expression(&stmt.test, self.unresolved_ctxt) {
          self.side_effects_span = Some(stmt.span);
        }
      }
      Stmt::While(stmt) => {
        if !is_pure_expression(&stmt.test, self.unresolved_ctxt) {
          self.side_effects_span = Some(stmt.span);
        }
      }
      Stmt::DoWhile(stmt) => {
        if !is_pure_expression(&stmt.test, self.unresolved_ctxt) {
          self.side_effects_span = Some(stmt.span);
        }
      }
      Stmt::For(stmt) => {
        let pure_init = match stmt.init {
          Some(ref init) => match init {
            VarDeclOrExpr::VarDecl(decl) => is_pure_var_decl(decl, self.unresolved_ctxt),
            VarDeclOrExpr::Expr(expr) => is_pure_expression(expr, self.unresolved_ctxt),
          },
          None => true,
        };

        if !pure_init {
          self.side_effects_span = Some(stmt.span);
          return;
        }

        let pure_test = match stmt.test {
          Some(box ref test) => is_pure_expression(test, self.unresolved_ctxt),
          None => true,
        };

        if !pure_test {
          self.side_effects_span = Some(stmt.span);
          return;
        }

        let pure_update = match stmt.update {
          Some(ref expr) => is_pure_expression(expr, self.unresolved_ctxt),
          None => true,
        };

        if !pure_update {
          self.side_effects_span = Some(stmt.span);
        }
      }
      Stmt::Expr(stmt) => {
        if !is_pure_expression(&stmt.expr, self.unresolved_ctxt) {
          self.side_effects_span = Some(stmt.span);
        }
      }
      Stmt::Switch(stmt) => {
        if !is_pure_expression(&stmt.discriminant, self.unresolved_ctxt) {
          self.side_effects_span = Some(stmt.span);
        }
      }
      Stmt::Decl(stmt) => {
        if !is_pure_decl(stmt, self.unresolved_ctxt) {
          self.side_effects_span = Some(stmt.span());
        }
      }
      Stmt::Empty(_) => {}
      Stmt::Labeled(_) => {}
      Stmt::Block(_) => {}
      _ => self.side_effects_span = Some(ele.span()),
    };
  }
}

pub fn is_pure_expression(expr: &Expr, unresolved_ctxt: SyntaxContext) -> bool {
  !expr.may_have_side_effects(&ExprCtx {
    unresolved_ctxt,
    is_unresolved_ref_safe: false,
  })
}

pub fn is_pure_decl(stmt: &Decl, unresolved_ctxt: SyntaxContext) -> bool {
  match stmt {
    Decl::Class(class) => is_pure_class(&class.class, unresolved_ctxt),
    Decl::Fn(_) => true,
    Decl::Var(var) => is_pure_var_decl(var, unresolved_ctxt),
    Decl::Using(_) => false,
    Decl::TsInterface(_) => unreachable!(),
    Decl::TsTypeAlias(_) => unreachable!(),

    Decl::TsEnum(_) => unreachable!(),
    Decl::TsModule(_) => unreachable!(),
  }
}

pub fn is_pure_class(class: &Class, unresolved_ctxt: SyntaxContext) -> bool {
  if let Some(ref super_class) = class.super_class {
    if !is_pure_expression(super_class, unresolved_ctxt) {
      return false;
    }
  }
  let is_pure_key = |key: &PropName| -> bool {
    match key {
      PropName::BigInt(_) | PropName::Ident(_) | PropName::Str(_) | PropName::Num(_) => true,
      PropName::Computed(ref computed) => is_pure_expression(&computed.expr, unresolved_ctxt),
    }
  };

  class.body.iter().all(|item| -> bool {
    match item {
      ClassMember::Constructor(cons) => is_pure_key(&cons.key),
      ClassMember::Method(method) => is_pure_key(&method.key),
      ClassMember::PrivateMethod(method) => {
        is_pure_expression(&Expr::PrivateName(method.key.clone()), unresolved_ctxt)
      }
      ClassMember::ClassProp(prop) => {
        is_pure_key(&prop.key)
          && (!prop.is_static
            || if let Some(ref value) = prop.value {
              is_pure_expression(value, unresolved_ctxt)
            } else {
              true
            })
      }
      ClassMember::PrivateProp(prop) => {
        is_pure_expression(&Expr::PrivateName(prop.key.clone()), unresolved_ctxt)
          && (!prop.is_static
            || if let Some(ref value) = prop.value {
              is_pure_expression(value, unresolved_ctxt)
            } else {
              true
            })
      }
      ClassMember::TsIndexSignature(_) => unreachable!(),
      ClassMember::Empty(_) => true,
      ClassMember::StaticBlock(_) => true,
      ClassMember::AutoAccessor(_) => true,
    }
  })
}

fn is_pure_var_decl(var: &VarDecl, unresolved_ctxt: SyntaxContext) -> bool {
  var.decls.iter().all(|decl| {
    if let Some(ref init) = decl.init {
      is_pure_expression(init, unresolved_ctxt)
    } else {
      true
    }
  })
}

fn is_import_decl(module_item: &ModuleItem) -> bool {
  matches!(module_item, ModuleItem::ModuleDecl(ModuleDecl::Import(_)))
}

pub trait ClassKey {
  fn class_key(&self) -> Option<&PropName>;
}

impl ClassKey for ClassMember {
  fn class_key(&self) -> Option<&PropName> {
    match self {
      ClassMember::Constructor(c) => Some(&c.key),
      ClassMember::Method(m) => Some(&m.key),
      ClassMember::PrivateMethod(_) => None,
      ClassMember::ClassProp(c) => Some(&c.key),
      ClassMember::PrivateProp(_) => None,
      ClassMember::TsIndexSignature(_) => unreachable!(),
      ClassMember::Empty(_) => None,
      ClassMember::StaticBlock(_) => None,
      ClassMember::AutoAccessor(a) => match a.key {
        Key::Private(_) => None,
        Key::Public(ref public) => Some(public),
      },
    }
  }
}

#[derive(Debug, Default)]
pub struct SideEffectsFlagPlugin;

#[async_trait]
impl Plugin for SideEffectsFlagPlugin {
  async fn optimize_dependencies(&self, compilation: &mut Compilation) -> Result<Option<()>> {
    let mg = &mut compilation.module_graph;
    // SAFETY: this method will not modify the map, and we can guarantee there is no other
    // thread access the map at the same time.
    let module_identifier_to_module = std::mem::take(&mut mg.module_identifier_to_module);
    for (mi, module) in module_identifier_to_module.iter() {
      let mut module_chain = HashSet::default();
      let side_effects_state = module.get_side_effects_connection_state(&mg, &mut module_chain);
      if side_effects_state != rspack_core::ConnectionState::Bool(false) {
        continue;
      }
      let cur_exports_info_id = mg.get_exports_info(mi).id;

      let incomming_connections = mg.get_incoming_connections_cloned(module);
      for con in incomming_connections {
        let dep = match mg.dependency_by_id(&con.dependency_id) {
          Some(dep) => dep,
          None => continue,
        };
        let dep_id = *dep.id();
        let is_reexport = dep
          .downcast_ref::<HarmonyExportImportedSpecifierDependency>()
          .is_some();
        let is_valid_import_specifier_dep = if let Some(import_specifier_dep) =
          dep.downcast_ref::<HarmonyImportSpecifierDependency>()
        {
          !import_specifier_dep.namespace_object_as_context
        } else {
          false
        };
        if !is_reexport && !is_valid_import_specifier_dep {
          continue;
        }
        if let Some(name) = dep
          .downcast_ref::<HarmonyExportImportedSpecifierDependency>()
          .and_then(|dep| dep.name.clone())
        {
          let export_info_id = mg.get_export_info(
            con
              .original_module_identifier
              .expect("should have original_module_identifier"),
            &name,
          );
          // TODO:
          export_info_id.move_target(
            mg,
            Box::new(|target: &ResolvedExportInfoTarget, mg: &ModuleGraph| {
              mg.module_by_identifier(&target.module)
                .expect("should have module graph")
                .get_side_effects_connection_state(mg, &mut HashSet::default())
                == ConnectionState::Bool(false)
            }),
            Box::new(
              move |target: &ResolvedExportInfoTarget, mg: &mut ModuleGraph| {
                mg.update_module(&dep_id, target.module);
                // TODO: Explain https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/optimize/SideEffectsFlagPlugin.js#L303-L306
                let ids = dep_id.get_ids(mg);
                let processed_ids = target
                  .exports
                  .as_ref()
                  .map(|item| {
                    let mut ret = Vec::from_iter(item.iter().cloned());
                    ret.extend_from_slice(&ids[1..]);
                    ret
                  })
                  .unwrap_or_else(|| ids[1..].iter().map(|item| item.clone()).collect::<Vec<_>>());
                dep_id.set_ids(processed_ids, mg);
                mg.connection_by_dependency(&dep_id).cloned()
              },
            ),
          );
          continue;
        }
        // get dependency by id instead directly use it here because we don't  by
        let ids = dep_id.get_ids(mg);
        if ids.len() > 0 {
          let export_info_id = cur_exports_info_id.get_export_info(&ids[0], mg);
          let target = export_info_id.get_target(
            mg,
            Some(Box::new(
              |target: &ResolvedExportInfoTarget, mg: &ModuleGraph| {
                mg.module_by_identifier(&target.module)
                  .expect("should have module graph")
                  .get_side_effects_connection_state(mg, &mut HashSet::default())
                  == ConnectionState::Bool(false)
              },
            )),
          );
          let target = match target {
            Some(target) => target,
            None => continue,
          };
          mg.update_module(&dep_id, target.module);
          // TODO: Explain https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/optimize/SideEffectsFlagPlugin.js#L303-L306
          let processed_ids = target
            .exports
            .map(|mut item| {
              item.extend_from_slice(&ids[1..]);
              item
            })
            .unwrap_or_else(|| ids[1..].iter().map(|item| item.clone()).collect::<Vec<_>>());
          dep_id.set_ids(processed_ids, mg);
        }
      }
    }
    mg.module_identifier_to_module = module_identifier_to_module;
    Ok(None)
  }
}
