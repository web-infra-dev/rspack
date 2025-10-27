use std::{fmt::Debug, rc::Rc, sync::LazyLock};

use rayon::prelude::*;
use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_core::{
  BoxModule, Compilation, CompilationOptimizeDependencies, ConnectionState, DependencyExtraMeta,
  DependencyId, FactoryMeta, Logger, MaybeDynamicTargetExportInfo, ModuleFactoryCreateData,
  ModuleGraph, ModuleGraphConnection, ModuleIdentifier, NormalModuleCreateData,
  NormalModuleFactoryModule, Plugin, PrefetchExportsInfoMode, ResolvedExportInfoTarget,
  SideEffectsBailoutItemWithSpan, SideEffectsDoOptimize, SideEffectsDoOptimizeMoveTarget,
  incremental::{self, IncrementalPasses, Mutation},
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_paths::{AssertUtf8, Utf8Path};
use sugar_path::SugarPath;
use swc_core::{
  common::{GLOBALS, Spanned, SyntaxContext, comments, comments::Comments},
  ecma::{
    ast::*,
    utils::{ExprCtx, ExprExt},
    visit::{Visit, VisitWith, noop_visit_type},
  },
};

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

        let pure_test = match &stmt.test {
          Some(test) => is_pure_expression(test, self.unresolved_ctxt, self.comments),
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
  expr: &Expr,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&dyn Comments>,
) -> bool {
  let Expr::Call(call_expr) = expr else {
    unreachable!();
  };
  let callee = &call_expr.callee;
  let pure_flag = comments
    .and_then(|comments| {
      if let Some(comment_list) = comments.get_leading(callee.span().lo) {
        return Some(comment_list.iter().any(|comment| {
          comment.kind == comments::CommentKind::Block && PURE_COMMENTS.is_match(&comment.text)
        }));
      }
      None
    })
    .unwrap_or(false);
  if !pure_flag {
    !expr.may_have_side_effects(ExprCtx {
      unresolved_ctxt,
      in_strict: false,
      is_unresolved_ref_safe: false,
      remaining_depth: 4,
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
  ) -> bool {
    match expr {
      Expr::Call(_) => is_pure_call_expr(expr, unresolved_ctxt, comments),
      Expr::Paren(_) => unreachable!(),
      _ => !expr.may_have_side_effects(ExprCtx {
        unresolved_ctxt,
        is_unresolved_ref_safe: true,
        in_strict: false,
        remaining_depth: 4,
      }),
    }
  }
  _is_pure_expression(expr, unresolved_ctxt, comments)
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
    ClassMember::PrivateProp(prop) => {
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
  if let Some(ref super_class) = class.super_class
    && !is_pure_expression(super_class, unresolved_ctxt, comments)
  {
    return false;
  }
  let is_pure_key = |key: &PropName| -> bool {
    match key {
      PropName::BigInt(_) | PropName::Ident(_) | PropName::Str(_) | PropName::Num(_) => true,
      PropName::Computed(computed) => is_pure_expression(&computed.expr, unresolved_ctxt, comments),
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

#[plugin_hook(NormalModuleFactoryModule for SideEffectsFlagPlugin,tracing=false)]
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
  let Some(resource_path) = resource_data.path() else {
    return Ok(());
  };
  let Some(description) = resource_data.description() else {
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

#[plugin_hook(CompilationOptimizeDependencies for SideEffectsFlagPlugin,tracing=false)]
async fn optimize_dependencies(&self, compilation: &mut Compilation) -> Result<Option<bool>> {
  let logger = compilation.get_logger("rspack.SideEffectsFlagPlugin");
  let start = logger.time("update connections");

  let mut side_effects_optimize_artifact = if compilation
    .incremental
    .passes_enabled(IncrementalPasses::SIDE_EFFECTS)
  {
    std::mem::take(&mut compilation.side_effects_optimize_artifact)
  } else {
    Default::default()
  };
  let module_graph = compilation.get_module_graph();

  let all_modules = module_graph.modules();

  let side_effects_state_map: IdentifierMap<ConnectionState> = all_modules
    .par_iter()
    .map(|(module_identifier, module)| {
      (
        *module_identifier,
        module.get_side_effects_connection_state(
          &module_graph,
          &compilation.module_graph_cache_artifact,
          &mut Default::default(),
          &mut Default::default(),
        ),
      )
    })
    .collect();

  let inner_start = logger.time("prepare connections");
  let modules: IdentifierSet = if let Some(mutations) = compilation
    .incremental
    .mutations_read(IncrementalPasses::SIDE_EFFECTS)
    && !side_effects_optimize_artifact.is_empty()
  {
    side_effects_optimize_artifact.retain(|dependency_id, do_optimize| {
      let dep_exist = module_graph
        .connection_by_dependency_id(dependency_id)
        .is_some();
      let target_module_exist = module_graph
        .module_by_identifier(&do_optimize.target_module)
        .is_some();
      dep_exist && target_module_exist
    });

    fn affected_incoming_modules(
      module: &ModuleIdentifier,
      module_graph: &ModuleGraph,
      modules: &mut IdentifierSet,
    ) {
      for connection in module_graph.get_incoming_connections(module) {
        let Some(original_module) = connection.original_module_identifier else {
          continue;
        };
        if modules.contains(&original_module) {
          continue;
        }
        let Some(dep) = module_graph.dependency_by_id(&connection.dependency_id) else {
          continue;
        };
        if dep.is::<ESMExportImportedSpecifierDependency>() && modules.insert(original_module) {
          affected_incoming_modules(&original_module, module_graph, modules);
        }
      }
    }

    let modules: IdentifierSet = mutations.iter().fold(
      IdentifierSet::default(),
      |mut modules, mutation| match mutation {
        Mutation::ModuleAdd { module } | Mutation::ModuleUpdate { module } => {
          if modules.insert(*module) {
            affected_incoming_modules(module, &module_graph, &mut modules);
          }
          modules.extend(
            module_graph
              .get_outgoing_connections(module)
              .map(|connection| *connection.module_identifier()),
          );
          modules
        }
        _ => modules,
      },
    );

    tracing::debug!(target: incremental::TRACING_TARGET, passes = %IncrementalPasses::SIDE_EFFECTS, %mutations, ?modules);
    let logger = compilation.get_logger("rspack.incremental.sideEffects");
    logger.log(format!(
      "{} modules are affected, {} in total",
      modules.len(),
      all_modules.len()
    ));

    modules
  } else {
    all_modules.keys().copied().collect()
  };
  logger.time_end(inner_start);

  let inner_start = logger.time("find optimizable connections");
  let dep_optimize_info = modules
    .par_iter()
    .filter(|module| side_effects_state_map[module] == ConnectionState::Active(false))
    .flat_map(|module| {
      module_graph
        .get_incoming_connections(module)
        .collect::<Vec<_>>()
    })
    .map(|connection| {
      (
        connection.dependency_id,
        can_optimize_connection(connection, &side_effects_state_map, &module_graph),
      )
    })
    .collect::<Vec<_>>();
  for (dep_id, can_optimize) in dep_optimize_info {
    if let Some(do_optimize) = can_optimize {
      side_effects_optimize_artifact.insert(dep_id, do_optimize);
    } else {
      side_effects_optimize_artifact.remove(&dep_id);
    }
  }
  logger.time_end(inner_start);

  let mut do_optimizes = side_effects_optimize_artifact.clone();

  let inner_start = logger.time("do optimize connections");
  let mut do_optimized_count = 0;
  while !do_optimizes.is_empty() {
    do_optimized_count += do_optimizes.len();

    let mut module_graph = compilation.get_module_graph_mut();
    let new_connections: Vec<_> = do_optimizes
      .into_iter()
      .map(|(dependency, do_optimize)| {
        do_optimize_connection(dependency, do_optimize, &mut module_graph)
      })
      .collect();

    let module_graph = compilation.get_module_graph();
    do_optimizes = new_connections
      .into_par_iter()
      .filter(|(_, module)| side_effects_state_map[module] == ConnectionState::Active(false))
      .filter_map(|(connection, _)| module_graph.connection_by_dependency_id(&connection))
      .filter_map(|connection| {
        can_optimize_connection(connection, &side_effects_state_map, &module_graph)
          .map(|i| (connection.dependency_id, i))
      })
      .collect();
  }
  logger.time_end(inner_start);

  compilation.side_effects_optimize_artifact = side_effects_optimize_artifact;

  logger.time_end(start);
  logger.log(format!("optimized {do_optimized_count} connections"));
  Ok(None)
}

#[tracing::instrument(skip_all)]
fn do_optimize_connection(
  dependency: DependencyId,
  do_optimize: SideEffectsDoOptimize,
  module_graph: &mut ModuleGraph,
) -> (DependencyId, ModuleIdentifier) {
  let SideEffectsDoOptimize {
    ids,
    target_module,
    need_move_target,
  } = do_optimize;
  module_graph.do_update_module(&dependency, &target_module);
  module_graph.set_dependency_extra_meta(
    dependency,
    DependencyExtraMeta {
      ids,
      explanation: Some("(skipped side-effect-free modules)"),
    },
  );
  if let Some(SideEffectsDoOptimizeMoveTarget {
    export_info,
    target_export,
  }) = need_move_target
  {
    export_info
      .as_data_mut(module_graph)
      .do_move_target(dependency, target_export);
  }
  (dependency, target_module)
}

#[tracing::instrument("can_optimize_connection", level = "trace", skip_all)]
fn can_optimize_connection(
  connection: &ModuleGraphConnection,
  side_effects_state_map: &IdentifierMap<ConnectionState>,
  module_graph: &ModuleGraph,
) -> Option<SideEffectsDoOptimize> {
  let original_module = connection.original_module_identifier?;
  let dependency_id = connection.dependency_id;
  let dep = module_graph.dependency_by_id(&dependency_id)?;

  if let Some(dep) = dep.downcast_ref::<ESMExportImportedSpecifierDependency>()
    && let Some(name) = &dep.name
  {
    let exports_info =
      module_graph.get_prefetched_exports_info(&original_module, PrefetchExportsInfoMode::Default);
    let export_info = exports_info.get_export_info_without_mut_module_graph(name);

    let target = export_info.can_move_target(
      module_graph,
      Rc::new(|target: &ResolvedExportInfoTarget| {
        side_effects_state_map[&target.module] == ConnectionState::Active(false)
      }),
    )?;
    if !module_graph.can_update_module(&dependency_id, &target.module) {
      return None;
    }

    let ids = dep.get_ids(module_graph);
    let processed_ids = target
      .export
      .as_ref()
      .map(|item| {
        let mut ret = Vec::from_iter(item.iter().cloned());
        ret.extend_from_slice(ids.get(1..).unwrap_or_default());
        ret
      })
      .unwrap_or_else(|| ids.get(1..).unwrap_or_default().to_vec());
    let need_move_target = match export_info {
      MaybeDynamicTargetExportInfo::Static(export_info) => Some(SideEffectsDoOptimizeMoveTarget {
        export_info: export_info.id(),
        target_export: target.export,
      }),
      MaybeDynamicTargetExportInfo::Dynamic { .. } => None,
    };

    return Some(SideEffectsDoOptimize {
      ids: processed_ids,
      target_module: target.module,
      need_move_target,
    });
  }

  if let Some(dep) = dep.downcast_ref::<ESMImportSpecifierDependency>()
    && !dep.namespace_object_as_context
    && let ids = dep.get_ids(module_graph)
    && !ids.is_empty()
  {
    let exports_info = module_graph.get_prefetched_exports_info(
      connection.module_identifier(),
      PrefetchExportsInfoMode::Default,
    );
    let export_info = exports_info.get_export_info_without_mut_module_graph(&ids[0]);

    let target = export_info.get_target(
      module_graph,
      Rc::new(|target: &ResolvedExportInfoTarget| {
        side_effects_state_map[&target.module] == ConnectionState::Active(false)
      }),
    )?;
    if !module_graph.can_update_module(&dependency_id, &target.module) {
      return None;
    }

    let processed_ids = target
      .export
      .map(|mut item| {
        item.extend_from_slice(&ids[1..]);
        item
      })
      .unwrap_or_else(|| ids[1..].to_vec());

    return Some(SideEffectsDoOptimize {
      ids: processed_ids,
      target_module: target.module,
      need_move_target: None,
    });
  }

  None
}

impl Plugin for SideEffectsFlagPlugin {
  fn name(&self) -> &'static str {
    "SideEffectsFlagPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .normal_module_factory_hooks
      .module
      .tap(nmf_module::new(self));
    ctx
      .compilation_hooks
      .optimize_dependencies
      .tap(optimize_dependencies::new(self));
    Ok(())
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
