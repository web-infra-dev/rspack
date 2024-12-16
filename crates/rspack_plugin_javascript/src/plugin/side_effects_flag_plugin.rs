use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::LazyLock;

use rspack_collections::IdentifierMap;
use rspack_collections::IdentifierSet;
use rspack_core::DependencyId;
use rspack_core::{
  BoxModule, Compilation, CompilationOptimizeDependencies, ConnectionState, FactoryMeta,
  ModuleFactoryCreateData, ModuleGraph, ModuleIdentifier, NormalModuleCreateData,
  NormalModuleFactoryModule, Plugin, ResolvedExportInfoTarget, SideEffectsBailoutItemWithSpan,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_paths::AssertUtf8;
use rspack_paths::Utf8Path;
use rustc_hash::FxHashSet;
use sugar_path::SugarPath;
use swc_core::common::comments::Comments;
use swc_core::common::{comments, Span, Spanned, SyntaxContext, GLOBALS};
use swc_core::ecma::ast::*;
use swc_core::ecma::utils::{ExprCtx, ExprExt};
use swc_core::ecma::visit::{noop_visit_type, Visit, VisitWith};

use crate::dependency::{ESMExportImportedSpecifierDependency, ESMImportSpecifierDependency};

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

fn get_side_effects_from_package_json(side_effects: SideEffects, relative_path: &Utf8Path) -> bool {
  match side_effects {
    SideEffects::Bool(s) => s,
    SideEffects::String(s) => glob_match_with_normalized_pattern(&s, relative_path.as_str()),
    SideEffects::Array(patterns) => patterns
      .iter()
      .any(|pattern| glob_match_with_normalized_pattern(pattern, relative_path.as_str())),
  }
}

fn glob_match_with_normalized_pattern(pattern: &str, string: &str) -> bool {
  let trim_start = pattern.trim_start_matches("./");
  let normalized_glob = if trim_start.contains('/') {
    trim_start.to_string()
  } else {
    String::from("**/") + trim_start
  };
  fast_glob::glob_match(&normalized_glob, string.trim_start_matches("./"))
}

pub struct SideEffectsFlagPluginVisitor<'a> {
  unresolved_ctxt: SyntaxContext,
  pub side_effects_item: Option<SideEffectsBailoutItemWithSpan>,
  is_top_level: bool,
  comments: Option<&'a dyn Comments>,
}

impl Debug for SideEffectsFlagPluginVisitor<'_> {
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
  pub fn new(mark_info: SyntaxContextInfo, comments: Option<&'a dyn Comments>) -> Self {
    Self {
      unresolved_ctxt: mark_info.unresolved_ctxt,
      side_effects_item: None,
      is_top_level: true,
      comments,
    }
  }
}

impl Visit for SideEffectsFlagPluginVisitor<'_> {
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

impl SideEffectsFlagPluginVisitor<'_> {
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

static PURE_COMMENTS: LazyLock<regex::Regex> =
  LazyLock::new(|| regex::Regex::new("^\\s*(#|@)__PURE__\\s*$").expect("Should create the regex"));

fn is_pure_call_expr(
  call_expr: &CallExpr,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&dyn Comments>,
  paren_spans: &mut Vec<Span>,
) -> bool {
  let callee = &call_expr.callee;
  let pure_flag = comments
    .and_then(|comments| {
      paren_spans.push(callee.span());
      while let Some(span) = paren_spans.pop() {
        if let Some(comment_list) = comments.get_leading(span.lo)
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
      in_strict: false,
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

pub fn is_pure_pat<'a>(
  pat: &'a Pat,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&'a dyn Comments>,
) -> bool {
  match pat {
    Pat::Ident(_) => true,
    Pat::Array(array_pat) => array_pat.elems.iter().all(|ele| {
      if let Some(pat) = ele {
        is_pure_pat(pat, unresolved_ctxt, comments)
      } else {
        true
      }
    }),
    Pat::Rest(_) => true,
    Pat::Invalid(_) | Pat::Assign(_) | Pat::Object(_) => false,
    Pat::Expr(expr) => is_pure_expression(expr, unresolved_ctxt, comments),
  }
}

pub fn is_pure_function<'a>(
  function: &'a Function,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&'a dyn Comments>,
) -> bool {
  if !function
    .params
    .iter()
    .all(|param| is_pure_pat(&param.pat, unresolved_ctxt, comments))
  {
    return false;
  }

  true
}

pub fn is_pure_expression<'a>(
  expr: &'a Expr,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&'a dyn Comments>,
) -> bool {
  pub fn _is_pure_expression<'a>(
    expr: &'a Expr,
    unresolved_ctxt: SyntaxContext,
    comments: Option<&'a dyn Comments>,
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
        in_strict: false,
      }),
    }
  }
  _is_pure_expression(expr, unresolved_ctxt, comments, &mut vec![])
}

pub fn is_pure_class_member<'a>(
  member: &'a ClassMember,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&'a dyn Comments>,
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
  comments: Option<&dyn Comments>,
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
  comments: Option<&dyn Comments>,
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
  comments: Option<&'a dyn Comments>,
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
  let Some(resource_path) = &resource_data.resource_path else {
    return Ok(());
  };
  let Some(description) = resource_data.resource_description.as_ref() else {
    return Ok(());
  };
  let package_path = description.path();
  let Some(side_effects) = SideEffects::from_description(description.json()) else {
    return Ok(());
  };
  let relative_path = resource_path
    .as_std_path()
    .relative(package_path)
    .assert_utf8();
  let has_side_effects = get_side_effects_from_package_json(side_effects, relative_path.as_path());
  module.set_factory_meta(FactoryMeta {
    side_effect_free: Some(!has_side_effects),
  });
  Ok(())
}

#[plugin_hook(CompilationOptimizeDependencies for SideEffectsFlagPlugin)]
fn optimize_dependencies(&self, compilation: &mut Compilation) -> Result<Option<bool>> {
  // TODO: use affected module optimization
  let mut modules: IdentifierSet = compilation
    .get_module_graph()
    .modules()
    .keys()
    .copied()
    .collect();
  let mut new_connections = Default::default();
  let cache = Rc::new(RefCell::new(Default::default()));
  for module in modules.clone() {
    optimize_incoming_connections(
      module,
      &mut modules,
      &mut new_connections,
      compilation,
      cache.clone(),
    );
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
    _options: &rspack_core::CompilerOptions,
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

#[tracing::instrument(skip_all, fields(module = ?module_identifier))]
fn optimize_incoming_connections(
  module_identifier: ModuleIdentifier,
  to_be_optimized: &mut IdentifierSet,
  new_connections: &mut IdentifierMap<FxHashSet<DependencyId>>,
  compilation: &mut Compilation,
  cache: Rc<RefCell<IdentifierMap<ConnectionState>>>,
) {
  if !to_be_optimized.remove(&module_identifier) {
    return;
  }
  let module_graph = compilation.get_module_graph();
  let Some(module) = module_graph.module_by_identifier(&module_identifier) else {
    return;
  };

  let mut cache_inner = cache.borrow_mut();
  let side_effects_state = if let Some(state) = cache_inner.get(&module_identifier) {
    *state
  } else {
    let state =
      module.get_side_effects_connection_state(&module_graph, &mut IdentifierSet::default());
    cache_inner.insert(module_identifier, state);
    state
  };
  drop(cache_inner);

  if side_effects_state != rspack_core::ConnectionState::Bool(false) {
    return;
  }
  let incoming_connections = module_graph
    .module_graph_module_by_identifier(&module_identifier)
    .map(|mgm| mgm.incoming_connections().clone())
    .unwrap_or_default();
  for &dep_id in &incoming_connections {
    optimize_incoming_connection(
      dep_id,
      module_identifier,
      to_be_optimized,
      new_connections,
      compilation,
      cache.clone(),
    );
  }
  // It is possible to add additional new connections when optimizing module's incoming connections
  while let Some(connections) = new_connections.remove(&module_identifier) {
    for new_dep_id in connections {
      optimize_incoming_connection(
        new_dep_id,
        module_identifier,
        to_be_optimized,
        new_connections,
        compilation,
        cache.clone(),
      );
    }
  }
}

fn optimize_incoming_connection(
  dependency_id: DependencyId,
  module_identifier: ModuleIdentifier,
  to_be_optimized: &mut IdentifierSet,
  new_connections: &mut IdentifierMap<FxHashSet<DependencyId>>,
  compilation: &mut Compilation,
  cache: Rc<RefCell<IdentifierMap<ConnectionState>>>,
) {
  let module_graph = compilation.get_module_graph();
  let connection = module_graph
    .connection_by_dependency_id(&dependency_id)
    .expect("should have connection");
  let Some(dep) = module_graph.dependency_by_id(&dependency_id) else {
    return;
  };
  let is_reexport = dep
    .downcast_ref::<ESMExportImportedSpecifierDependency>()
    .is_some();
  let is_valid_import_specifier_dep = dep
    .downcast_ref::<ESMImportSpecifierDependency>()
    .map(|import_specifier_dep| !import_specifier_dep.namespace_object_as_context)
    .unwrap_or_default();
  if !is_reexport && !is_valid_import_specifier_dep {
    return;
  }
  let Some(origin_module) = connection.original_module_identifier else {
    return;
  };
  // For the best optimization results, connection.origin_module must optimize before connection.module
  // See: https://github.com/webpack/webpack/pull/17595
  optimize_incoming_connections(
    origin_module,
    to_be_optimized,
    new_connections,
    compilation,
    cache.clone(),
  );
  do_optimize_incoming_connection(
    dependency_id,
    module_identifier,
    origin_module,
    new_connections,
    compilation,
    cache.clone(),
  );
}

#[tracing::instrument(skip_all, fields(origin = ?origin_module, module = ?module_identifier))]
fn do_optimize_incoming_connection(
  dependency_id: DependencyId,
  module_identifier: ModuleIdentifier,
  origin_module: ModuleIdentifier,
  new_connections: &mut IdentifierMap<FxHashSet<DependencyId>>,
  compilation: &mut Compilation,
  cache: Rc<RefCell<IdentifierMap<ConnectionState>>>,
) {
  if let Some(connections) = new_connections.get_mut(&module_identifier) {
    connections.remove(&dependency_id);
  }
  let mut module_graph = compilation.get_module_graph_mut();
  let dep = module_graph
    .dependency_by_id(&dependency_id)
    .expect("should have dep");
  if let Some(name) = dep
    .downcast_ref::<ESMExportImportedSpecifierDependency>()
    .and_then(|dep| dep.name.clone())
  {
    let export_info = module_graph.get_export_info(origin_module, &name);
    let cache_clone = cache.clone();
    let target = export_info.move_target(
      &mut module_graph,
      Rc::new(
        move |target: &ResolvedExportInfoTarget, mg: &mut ModuleGraph| {
          let mut cache = cache_clone.borrow_mut();
          let state = if let Some(state) = cache.get(&target.module) {
            *state
          } else {
            let state = mg
              .module_by_identifier(&target.module)
              .expect("should have module")
              .get_side_effects_connection_state(mg, &mut IdentifierSet::default());
            cache.insert(target.module, state);
            state
          };

          state == ConnectionState::Bool(false)
        },
      ),
      Arc::new(
        move |target: &ResolvedExportInfoTarget, mg: &mut ModuleGraph| {
          if !mg.update_module(&dependency_id, &target.module) {
            return None;
          }
          // TODO: Explain https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/optimize/SideEffectsFlagPlugin.js#L303-L306
          let ids = dependency_id.get_ids(mg);
          let processed_ids = target
            .export
            .as_ref()
            .map(|item| {
              let mut ret = Vec::from_iter(item.iter().cloned());
              ret.extend_from_slice(ids.get(1..).unwrap_or_default());
              ret
            })
            .unwrap_or_else(|| ids.get(1..).unwrap_or_default().to_vec());
          dependency_id.set_ids(processed_ids, mg);
          Some(dependency_id)
        },
      ),
    );
    if let Some(ResolvedExportInfoTarget {
      dependency, module, ..
    }) = target
    {
      new_connections
        .entry(module)
        .or_default()
        .insert(dependency);
    };
    return;
  }

  let ids = dependency_id.get_ids(&module_graph);
  if !ids.is_empty() {
    let cur_exports_info = module_graph.get_exports_info(&module_identifier);
    let export_info = cur_exports_info.get_export_info(&mut module_graph, &ids[0]);

    let cache = cache.clone();
    let target = export_info.get_target_mut(
      &mut module_graph,
      Rc::new(
        move |target: &ResolvedExportInfoTarget, mg: &mut ModuleGraph| {
          let mut cache = cache.borrow_mut();
          let state = if let Some(state) = cache.get(&target.module) {
            *state
          } else {
            let state = mg
              .module_by_identifier(&target.module)
              .expect("should have module graph")
              .get_side_effects_connection_state(mg, &mut IdentifierSet::default());
            cache.insert(target.module, state);
            state
          };
          state == ConnectionState::Bool(false)
        },
      ),
    );
    let Some(target) = target else {
      return;
    };

    if !module_graph.update_module(&dependency_id, &target.module) {
      return;
    };
    new_connections
      .entry(target.module)
      .or_default()
      .insert(dependency_id);
    // TODO: Explain https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/optimize/SideEffectsFlagPlugin.js#L303-L306
    let processed_ids = target
      .export
      .map(|mut item| {
        item.extend_from_slice(&ids[1..]);
        item
      })
      .unwrap_or_else(|| ids[1..].to_vec());
    dependency_id.set_ids(processed_ids, &mut module_graph);
  }
}

#[cfg(test)]
mod test_side_effects {
  use super::*;

  fn get_side_effects_from_package_json_helper(
    side_effects_config: Vec<&str>,
    relative_path: &str,
  ) -> bool {
    assert!(!side_effects_config.is_empty());
    let relative_path = Utf8Path::new(relative_path);
    let side_effects = if side_effects_config.len() > 1 {
      SideEffects::Array(
        side_effects_config
          .into_iter()
          .map(String::from)
          .collect::<Vec<_>>(),
      )
    } else {
      SideEffects::String((&side_effects_config[0]).to_string())
    };

    get_side_effects_from_package_json(side_effects, relative_path)
  }

  #[test]
  fn cases() {
    assert!(get_side_effects_from_package_json_helper(
      vec!["./src/**/*.js"],
      "./src/x/y/z.js"
    ));
    assert!(get_side_effects_from_package_json_helper(
      vec!["./src/index.js", "./src/selection/index.js"],
      "./src/selection/index.js"
    ));
    assert!(!get_side_effects_from_package_json_helper(
      vec!["./src/**/*.js"],
      "./x.js"
    ));
    assert!(get_side_effects_from_package_json_helper(
      vec!["./**/src/x/y/z.js"],
      "./src/x/y/z.js"
    ));
    // 				"./src/x/y/z.js",
    // 				"./src/**/z.js",
    assert!(get_side_effects_from_package_json_helper(
      vec!["./src/**/z.js"],
      "./src/x/y/z.js"
    ));
    // 				"./src/x/y/z.js",
    // 				"./**/x/**/z.js",
    assert!(get_side_effects_from_package_json_helper(
      vec!["./**/x/**/z.js"],
      "./src/x/y/z.js"
    ));
    // 				"./src/x/y/z.js",
    // 				"./**/src/**",
    assert!(get_side_effects_from_package_json_helper(
      vec!["./**/src/**"],
      "./src/x/y/z.js"
    ));
    // 				"./src/x/y/z.js",
    // 				"./**/src/*",
    assert!(!get_side_effects_from_package_json_helper(
      vec!["./src/x/y/z.js"],
      "./**/src/*"
    ));
    // 				"./src/x/y/z.js",
    // 				"*.js",
    assert!(get_side_effects_from_package_json_helper(
      vec!["*.js"],
      "./src/x/y/z.js"
    ));
    // 				"./src/x/y/z.js",
    // 				"x/**/z.js",
    assert!(!get_side_effects_from_package_json_helper(
      vec!["./src/x/y/z.js"],
      "x/**/z.js"
    ));
    // 				"./src/x/y/z.js",
    // 				"src/**/z.js",
    assert!(get_side_effects_from_package_json_helper(
      vec!["./src/**/z.js"],
      "./src/x/y/z.js"
    ));
    // 				"./src/x/y/z.js",
    // 				"src/**/{x,y,z}.js",
    assert!(get_side_effects_from_package_json_helper(
      vec!["src/**/{x,y,z}.js"],
      "./src/x/y/z.js"
    ));
    // 				"./src/x/y/z.js",
    // 				"src/**/[x-z].js",
    assert!(get_side_effects_from_package_json_helper(
      vec!["./src/**/[x-z].js"],
      "./src/x/y/z.js"
    ));
    // 		const array = ["./src/**/*.js", "./dirty.js"];
    assert!(get_side_effects_from_package_json_helper(
      vec!["./src/**/*.js", "./dirty.js"],
      "./src/x/y/z.js"
    ));
    assert!(get_side_effects_from_package_json_helper(
      vec!["./src/**/*.js", "./dirty.js"],
      "./dirty.js"
    ));
    assert!(!get_side_effects_from_package_json_helper(
      vec!["./src/**/*.js", "./dirty.js"],
      "./clean.js"
    ));
    assert!(get_side_effects_from_package_json_helper(
      vec!["./src/**/*/z.js"],
      "./src/x/y/z.js"
    ));
  }
}
