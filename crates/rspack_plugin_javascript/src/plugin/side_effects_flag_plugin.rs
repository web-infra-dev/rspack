use std::collections::VecDeque;
use std::fmt::Debug;
use std::path::PathBuf;
use std::sync::Arc;

use once_cell::sync::Lazy;
use rspack_core::{
  BoxModule, Compilation, CompilationOptimizeDependencies, ConnectionState, FactoryMeta,
  ModuleFactoryCreateData, ModuleGraph, ModuleIdentifier, MutableModuleGraph,
  NormalModuleCreateData, NormalModuleFactoryModule, Plugin, ResolvedExportInfoTarget,
  SideEffectsBailoutItemWithSpan,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_identifier::IdentifierSet;
use rustc_hash::FxHashSet as HashSet;
use sugar_path::SugarPath;
// use rspack_core::Plugin;
// use rspack_error::Result;
use swc_core::common::{comments, Span, Spanned, SyntaxContext, GLOBALS};
use swc_core::ecma::ast::*;
use swc_core::ecma::utils::{ExprCtx, ExprExt};
use swc_core::ecma::visit::{noop_visit_type, Visit, VisitWith};
use swc_node_comments::SwcComments;

use crate::dependency::{
  HarmonyExportImportedSpecifierDependency, HarmonyImportSpecifierDependency,
};

#[derive(Clone, Debug)]
enum SideEffects {
  Bool(bool),
  String(String),
  Array(Vec<String>),
}

impl SideEffects {
  pub fn from_description(description: &serde_json::Value) -> Option<Self> {
    description.get("sideEffects").and_then(|value| {
      if let Some(b) = value.as_bool() {
        Some(SideEffects::Bool(b))
      } else if let Some(s) = value.as_str() {
        Some(SideEffects::String(s.to_owned()))
      } else if let Some(vec) = value.as_array() {
        let mut side_effects = vec![];
        for value in vec {
          if let Some(str) = value.as_str() {
            side_effects.push(str.to_string());
          } else {
            return None;
          }
        }
        Some(SideEffects::Array(side_effects))
      } else {
        None
      }
    })
  }
}

fn get_side_effects_from_package_json(side_effects: SideEffects, relative_path: PathBuf) -> bool {
  match side_effects {
    SideEffects::Bool(s) => s,
    SideEffects::String(s) => {
      glob_match_with_normalized_pattern(&s, &relative_path.to_string_lossy())
    }
    SideEffects::Array(patterns) => patterns
      .iter()
      .any(|pattern| glob_match_with_normalized_pattern(pattern, &relative_path.to_string_lossy())),
  }
}

fn glob_match_with_normalized_pattern(pattern: &str, string: &str) -> bool {
  let trim_start = pattern.trim_start_matches("./");
  let normalized_glob = if trim_start.contains('/') {
    trim_start.to_string()
  } else {
    String::from("**/") + trim_start
  };
  glob_match::glob_match(&normalized_glob, string.trim_start_matches("./"))
}

pub struct SideEffectsFlagPluginVisitor<'a> {
  unresolved_ctxt: SyntaxContext,
  pub side_effects_item: Option<SideEffectsBailoutItemWithSpan>,
  is_top_level: bool,
  comments: Option<&'a SwcComments>,
}

impl<'a> Debug for SideEffectsFlagPluginVisitor<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("SideEffectsFlagPluginVisitor")
      .field("unresolved_ctxt", &self.unresolved_ctxt)
      .field("side_effects_span", &self.side_effects_item)
      .field("is_top_level", &self.is_top_level)
      .finish()
  }
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

impl<'a> SideEffectsFlagPluginVisitor<'a> {
  pub fn new(mark_info: SyntaxContextInfo, comments: Option<&'a SwcComments>) -> Self {
    Self {
      unresolved_ctxt: mark_info.unresolved_ctxt,
      side_effects_item: None,
      is_top_level: true,
      comments,
    }
  }
}

impl<'a> Visit for SideEffectsFlagPluginVisitor<'a> {
  noop_visit_type!();
  fn visit_program(&mut self, node: &Program) {
    assert!(GLOBALS.is_set());
    node.visit_children_with(self);
  }

  fn visit_module(&mut self, node: &Module) {
    for module_item in &node.body {
      match module_item {
        ModuleItem::ModuleDecl(decl) => match decl {
          ModuleDecl::Import(_) => {}
          ModuleDecl::ExportDecl(decl) => decl.visit_with(self),
          // `export { foo } from 'mod'`
          // `export { foo as bar } from 'mod'`
          ModuleDecl::ExportNamed(_) => {}
          ModuleDecl::ExportDefaultDecl(decl) => {
            decl.visit_with(self);
          }
          ModuleDecl::ExportDefaultExpr(expr) => {
            if !is_pure_expression(&expr.expr, self.unresolved_ctxt, self.comments) {
              self.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
                expr.span,
                String::from("ExportDefaultExpr"),
              ));
            }
          }
          // export * from './x'
          ModuleDecl::ExportAll(_) => {}
          ModuleDecl::TsImportEquals(_) => unreachable!(),
          ModuleDecl::TsExportAssignment(_) => unreachable!(),
          ModuleDecl::TsNamespaceExport(_) => unreachable!(),
        },
        ModuleItem::Stmt(stmt) => stmt.visit_with(self),
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

  fn visit_export_decl(&mut self, node: &ExportDecl) {
    if !is_pure_decl(&node.decl, self.unresolved_ctxt, self.comments) {
      self.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
        node.decl.span(),
        String::from("Decl"),
      ));
    }
    node.visit_children_with(self);
  }
  fn visit_class_member(&mut self, node: &ClassMember) {
    if let Some(key) = node.class_key()
      && key.is_computed()
    {
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

impl<'a> SideEffectsFlagPluginVisitor<'a> {
  /// If we find a stmt that has side effects, we will skip the rest of the stmts.
  /// And mark the module as having side effects.
  fn analyze_stmt_side_effects(&mut self, ele: &Stmt) {
    if self.side_effects_item.is_some() {
      return;
    }
    match ele {
      Stmt::If(stmt) => {
        if !is_pure_expression(&stmt.test, self.unresolved_ctxt, self.comments) {
          self.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            stmt.span(),
            String::from("Statement"),
          ));
        }
      }
      Stmt::While(stmt) => {
        if !is_pure_expression(&stmt.test, self.unresolved_ctxt, self.comments) {
          self.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            stmt.span(),
            String::from("Statement"),
          ));
        }
      }
      Stmt::DoWhile(stmt) => {
        if !is_pure_expression(&stmt.test, self.unresolved_ctxt, self.comments) {
          self.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            stmt.span(),
            String::from("Statement"),
          ));
        }
      }
      Stmt::For(stmt) => {
        let pure_init = match stmt.init {
          Some(ref init) => match init {
            VarDeclOrExpr::VarDecl(decl) => {
              is_pure_var_decl(decl, self.unresolved_ctxt, self.comments)
            }
            VarDeclOrExpr::Expr(expr) => {
              is_pure_expression(expr, self.unresolved_ctxt, self.comments)
            }
          },
          None => true,
        };

        if !pure_init {
          self.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            stmt.span(),
            String::from("Statement"),
          ));
          return;
        }

        let pure_test = match stmt.test {
          Some(box ref test) => is_pure_expression(test, self.unresolved_ctxt, self.comments),
          None => true,
        };

        if !pure_test {
          self.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            stmt.span(),
            String::from("Statement"),
          ));
          return;
        }

        let pure_update = match stmt.update {
          Some(ref expr) => is_pure_expression(expr, self.unresolved_ctxt, self.comments),
          None => true,
        };

        if !pure_update {
          self.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            stmt.span(),
            String::from("Statement"),
          ));
        }
      }
      Stmt::Expr(stmt) => {
        if !is_pure_expression(&stmt.expr, self.unresolved_ctxt, self.comments) {
          self.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            stmt.span(),
            String::from("Statement"),
          ));
        }
      }
      Stmt::Switch(stmt) => {
        if !is_pure_expression(&stmt.discriminant, self.unresolved_ctxt, self.comments) {
          self.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            stmt.span(),
            String::from("Statement"),
          ));
        }
      }
      Stmt::Decl(stmt) => {
        if !is_pure_decl(stmt, self.unresolved_ctxt, self.comments) {
          self.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            stmt.span(),
            String::from("Statement"),
          ));
        }
      }
      Stmt::Empty(_) => {}
      Stmt::Labeled(_) => {}
      Stmt::Block(_) => {}
      _ => {
        self.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
          ele.span(),
          String::from("Statement"),
        ))
      }
    };
  }
}

static PURE_COMMENTS: Lazy<regex::Regex> =
  Lazy::new(|| regex::Regex::new("^\\s*(#|@)__PURE__\\s*$").expect("Should create the regex"));

fn is_pure_call_expr(
  call_expr: &CallExpr,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&SwcComments>,
  paren_spans: &mut Vec<Span>,
) -> bool {
  let callee = &call_expr.callee;
  let pure_flag = comments
    .and_then(|comments| {
      paren_spans.push(callee.span());
      // dbg!(&comments.leading, &paren_spans);
      while let Some(span) = paren_spans.pop() {
        if let Some(comment_list) = comments.leading.get(&span.lo)
          && let Some(last_comment) = comment_list.last()
          && last_comment.kind == comments::CommentKind::Block
        {
          // iterate through the parens and check if it contains pure comment
          if PURE_COMMENTS.is_match(&last_comment.text) {
            return Some(true);
          }
        }
      }
      None
    })
    .unwrap_or(false);
  if !pure_flag {
    let expr = Expr::Call(call_expr.clone());
    !expr.may_have_side_effects(&ExprCtx {
      unresolved_ctxt,
      is_unresolved_ref_safe: false,
    })
  } else {
    call_expr.args.iter().all(|arg| {
      if arg.spread.is_some() {
        false
      } else {
        is_pure_expression(&arg.expr, unresolved_ctxt, comments)
      }
    })
  }
}

pub fn is_pure_expression<'a>(
  expr: &'a Expr,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&'a SwcComments>,
) -> bool {
  pub fn _is_pure_expression<'a>(
    expr: &'a Expr,
    unresolved_ctxt: SyntaxContext,
    comments: Option<&'a SwcComments>,
    paren_spans: &mut Vec<Span>,
  ) -> bool {
    match expr {
      Expr::Call(call) => is_pure_call_expr(call, unresolved_ctxt, comments, paren_spans),
      Expr::Paren(par) => {
        paren_spans.push(par.span());
        let mut cur = par.expr.as_ref();
        while let Expr::Paren(paren) = cur {
          paren_spans.push(paren.span());
          cur = paren.expr.as_ref();
        }

        _is_pure_expression(cur, unresolved_ctxt, comments, paren_spans)
      }
      _ => !expr.may_have_side_effects(&ExprCtx {
        unresolved_ctxt,
        is_unresolved_ref_safe: true,
      }),
    }
  }
  _is_pure_expression(expr, unresolved_ctxt, comments, &mut vec![])
}

pub fn is_pure_class_member<'a>(
  member: &'a ClassMember,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&'a SwcComments>,
) -> bool {
  let is_key_pure = match member.class_key() {
    Some(PropName::Ident(_ident)) => true,
    Some(PropName::Str(_)) => true,
    Some(PropName::Num(_)) => true,
    Some(PropName::Computed(computed)) => {
      is_pure_expression(&computed.expr, unresolved_ctxt, comments)
    }
    Some(PropName::BigInt(_)) => true,
    None => true,
  };
  if !is_key_pure {
    return false;
  }
  let is_static = member.is_static();
  let is_value_pure = match member {
    ClassMember::Constructor(_) => true,
    ClassMember::Method(_) => true,
    ClassMember::PrivateMethod(_) => true,
    ClassMember::ClassProp(prop) => {
      if let Some(ref value) = prop.value {
        is_pure_expression(value, unresolved_ctxt, comments)
      } else {
        true
      }
    }
    ClassMember::PrivateProp(ref prop) => {
      if let Some(ref value) = prop.value {
        is_pure_expression(value, unresolved_ctxt, comments)
      } else {
        true
      }
    }
    ClassMember::TsIndexSignature(_) => unreachable!(),
    ClassMember::Empty(_) => true,
    ClassMember::StaticBlock(_) => false,
    ClassMember::AutoAccessor(_) => false,
  };
  if is_static && !is_value_pure {
    return false;
  }
  true
}

pub fn is_pure_decl(
  stmt: &Decl,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&SwcComments>,
) -> bool {
  match stmt {
    Decl::Class(class) => is_pure_class(&class.class, unresolved_ctxt, comments),
    Decl::Fn(_) => true,
    Decl::Var(var) => is_pure_var_decl(var, unresolved_ctxt, comments),
    Decl::Using(_) => false,
    Decl::TsInterface(_) => unreachable!(),
    Decl::TsTypeAlias(_) => unreachable!(),

    Decl::TsEnum(_) => unreachable!(),
    Decl::TsModule(_) => unreachable!(),
  }
}

pub fn is_pure_class(
  class: &Class,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&SwcComments>,
) -> bool {
  if let Some(ref super_class) = class.super_class {
    if !is_pure_expression(super_class, unresolved_ctxt, comments) {
      return false;
    }
  }
  let is_pure_key = |key: &PropName| -> bool {
    match key {
      PropName::BigInt(_) | PropName::Ident(_) | PropName::Str(_) | PropName::Num(_) => true,
      PropName::Computed(ref computed) => {
        is_pure_expression(&computed.expr, unresolved_ctxt, comments)
      }
    }
  };

  class.body.iter().all(|item| -> bool {
    match item {
      ClassMember::Constructor(_) => class.super_class.is_none(),
      ClassMember::Method(method) => is_pure_key(&method.key),
      ClassMember::PrivateMethod(method) => is_pure_expression(
        &Expr::PrivateName(method.key.clone()),
        unresolved_ctxt,
        comments,
      ),
      ClassMember::ClassProp(prop) => {
        is_pure_key(&prop.key)
          && (!prop.is_static
            || if let Some(ref value) = prop.value {
              is_pure_expression(value, unresolved_ctxt, comments)
            } else {
              true
            })
      }
      ClassMember::PrivateProp(prop) => {
        is_pure_expression(
          &Expr::PrivateName(prop.key.clone()),
          unresolved_ctxt,
          comments,
        ) && (!prop.is_static
          || if let Some(ref value) = prop.value {
            is_pure_expression(value, unresolved_ctxt, comments)
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

fn is_pure_var_decl<'a>(
  var: &'a VarDecl,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&'a SwcComments>,
) -> bool {
  var.decls.iter().all(|decl| {
    if let Some(ref init) = decl.init {
      is_pure_expression(init, unresolved_ctxt, comments)
    } else {
      true
    }
  })
}

pub trait ClassExt {
  fn class_key(&self) -> Option<&PropName>;
  fn is_static(&self) -> bool;
}

impl ClassExt for ClassMember {
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

  fn is_static(&self) -> bool {
    match self {
      ClassMember::Constructor(_cons) => false,
      ClassMember::Method(m) => m.is_static,
      ClassMember::PrivateMethod(m) => m.is_static,
      ClassMember::ClassProp(p) => p.is_static,
      ClassMember::PrivateProp(p) => p.is_static,
      ClassMember::TsIndexSignature(_) => unreachable!(),
      ClassMember::Empty(_) => false,
      ClassMember::StaticBlock(_) => true,
      ClassMember::AutoAccessor(a) => a.is_static,
    }
  }
}

#[plugin]
#[derive(Debug, Default)]
pub struct SideEffectsFlagPlugin;

#[plugin_hook(NormalModuleFactoryModule for SideEffectsFlagPlugin)]
async fn nmf_module(
  &self,
  _data: &mut ModuleFactoryCreateData,
  create_data: &mut NormalModuleCreateData,
  module: &mut BoxModule,
) -> Result<()> {
  if let Some(has_side_effects) = create_data.side_effects {
    module.set_factory_meta(FactoryMeta {
      side_effect_free: Some(!has_side_effects),
    });
    return Ok(());
  }
  let resource_data = &create_data.resource_resolve_data;
  let resource_path = &resource_data.resource_path;
  let Some(description) = resource_data.resource_description.as_ref() else {
    return Ok(());
  };
  let package_path = description.path();
  let Some(side_effects) = SideEffects::from_description(description.json()) else {
    return Ok(());
  };
  let relative_path = resource_path.relative(package_path);
  let has_side_effects = get_side_effects_from_package_json(side_effects, relative_path);
  module.set_factory_meta(FactoryMeta {
    side_effect_free: Some(!has_side_effects),
  });
  Ok(())
}

#[plugin_hook(CompilationOptimizeDependencies for SideEffectsFlagPlugin)]
fn optimize_dependencies(&self, compilation: &mut Compilation) -> Result<Option<bool>> {
  let entries = compilation.entry_modules();
  let level_order_module_identifier =
    get_level_order_module_ids(&compilation.get_module_graph(), entries);
  for module_identifier in level_order_module_identifier {
    let module_graph = compilation.get_module_graph();
    let mut module_chain = HashSet::default();
    // dbg!(&module_identifier);
    let Some(module) = module_graph.module_by_identifier(&module_identifier) else {
      continue;
    };
    let side_effects_state =
      module.get_side_effects_connection_state(&module_graph, &mut module_chain);
    if side_effects_state != rspack_core::ConnectionState::Bool(false) {
      continue;
    }
    let cur_exports_info_id = module_graph.get_exports_info(&module_identifier).id;

    let incoming_connections = module_graph
      .module_graph_module_by_identifier(&module_identifier)
      .map(|mgm| mgm.incoming_connections().clone())
      .unwrap_or_default();
    for con_id in incoming_connections {
      let mut module_graph = compilation.get_module_graph_mut();
      let con = module_graph
        .connection_by_connection_id(&con_id)
        .expect("should have connection");
      let Some(dep) = module_graph.dependency_by_id(&con.dependency_id) else {
        continue;
      };
      let dep_id = *dep.id();
      let is_reexport = dep
        .downcast_ref::<HarmonyExportImportedSpecifierDependency>()
        .is_some();
      let is_valid_import_specifier_dep = dep
        .downcast_ref::<HarmonyImportSpecifierDependency>()
        .map(|import_specifier_dep| !import_specifier_dep.namespace_object_as_context)
        .unwrap_or_default();
      if !is_reexport && !is_valid_import_specifier_dep {
        continue;
      }
      if let Some(name) = dep
        .downcast_ref::<HarmonyExportImportedSpecifierDependency>()
        .and_then(|dep| dep.name.clone())
      {
        let export_info_id = module_graph.get_export_info(
          con
            .original_module_identifier
            .expect("should have original_module_identifier"),
          &name,
        );
        export_info_id.move_target(
          &mut module_graph,
          Arc::new(|target: &ResolvedExportInfoTarget, mg: &ModuleGraph| {
            mg.module_by_identifier(&target.module)
              .expect("should have module")
              .get_side_effects_connection_state(mg, &mut HashSet::default())
              == ConnectionState::Bool(false)
          }),
          Arc::new(
            move |target: &ResolvedExportInfoTarget, mg: &mut ModuleGraph| {
              mg.update_module(&dep_id, &target.module);
              // TODO: Explain https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/optimize/SideEffectsFlagPlugin.js#L303-L306
              let ids = dep_id.get_ids(mg);
              let processed_ids = target
                .export
                .as_ref()
                .map(|item| {
                  let mut ret = Vec::from_iter(item.iter().cloned());
                  ret.extend_from_slice(ids.get(1..).unwrap_or_default());
                  ret
                })
                .unwrap_or_else(|| ids.get(1..).unwrap_or_default().to_vec());
              dep_id.set_ids(processed_ids, mg);
              mg.connection_by_dependency(&dep_id).map(|_| dep_id)
            },
          ),
        );
        continue;
      }

      let ids = dep_id.get_ids(&module_graph);

      if !ids.is_empty() {
        let export_info_id = cur_exports_info_id.get_export_info(&ids[0], &mut module_graph);

        let mut mga = MutableModuleGraph::new(&mut module_graph);
        let target = export_info_id.get_target(
          &mut mga,
          Some(Arc::new(
            |target: &ResolvedExportInfoTarget, mg: &ModuleGraph| {
              mg.module_by_identifier(&target.module)
                .expect("should have module graph")
                .get_side_effects_connection_state(mg, &mut HashSet::default())
                == ConnectionState::Bool(false)
            },
          )),
        );
        let Some(target) = target else {
          continue;
        };

        // dbg!(&mg.connection_by_dependency(&dep_id));
        module_graph.update_module(&dep_id, &target.module);
        // TODO: Explain https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/optimize/SideEffectsFlagPlugin.js#L303-L306
        let processed_ids = target
          .export
          .map(|mut item| {
            item.extend_from_slice(&ids[1..]);
            item
          })
          .unwrap_or_else(|| ids[1..].to_vec());

        // dbg!(&mg.connection_by_dependency(&dep_id));
        dep_id.set_ids(processed_ids, &mut module_graph);
      }
    }
  }
  Ok(None)
}

impl Plugin for SideEffectsFlagPlugin {
  fn name(&self) -> &'static str {
    "SideEffectsFlagPlugin"
  }

  fn apply(
    &self,
    ctx: rspack_core::PluginContext<&mut rspack_core::ApplyContext>,
    _options: &mut rspack_core::CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .normal_module_factory_hooks
      .module
      .tap(nmf_module::new(self));
    ctx
      .context
      .compilation_hooks
      .optimize_dependencies
      .tap(optimize_dependencies::new(self));
    Ok(())
  }
}

fn get_level_order_module_ids(mg: &ModuleGraph, entries: IdentifierSet) -> Vec<ModuleIdentifier> {
  let mut res = vec![];
  let mut visited = IdentifierSet::default();
  for entry in entries {
    let mut q = VecDeque::from_iter([entry]);
    while let Some(mi) = q.pop_front() {
      if visited.contains(&mi) {
        continue;
      } else {
        visited.insert(mi);
        res.push(mi);
      }
      for con in mg.get_outgoing_connections(&mi) {
        let mi = *con.module_identifier();
        q.push_back(mi);
      }
    }
  }

  res.sort_by(|a, b| {
    let ad = mg.get_depth(a);
    let bd = mg.get_depth(b);
    ad.cmp(&bd)
  });
  res
}
