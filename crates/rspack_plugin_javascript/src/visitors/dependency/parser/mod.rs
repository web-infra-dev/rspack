mod call_hooks_name;
mod walk;
mod walk_block_pre;
mod walk_pre;

use std::borrow::Cow;
use std::rc::Rc;

use bitflags::bitflags;
pub use call_hooks_name::CallHooksName;
use rspack_core::needs_refactor::WorkerSyntaxList;
use rspack_core::{
  AsyncDependenciesBlock, BoxDependency, BuildInfo, BuildMeta, DependencyTemplate,
  JavascriptParserOptions, ModuleIdentifier, ResourceData,
};
use rspack_core::{CompilerOptions, JavascriptParserUrl, ModuleType, SpanExt};
use rspack_error::miette::Diagnostic;
use rustc_hash::{FxHashMap, FxHashSet};
use swc_core::atoms::Atom;
use swc_core::common::comments::Comments;
use swc_core::common::util::take::Take;
use swc_core::common::{SourceFile, Span, Spanned};
use swc_core::ecma::ast::{
  ArrayPat, AssignPat, AssignTargetPat, CallExpr, Callee, MetaPropExpr, MetaPropKind, ObjectPat,
  ObjectPatProp, Pat, Program, Stmt, ThisExpr,
};
use swc_core::ecma::ast::{Expr, Ident, Lit, MemberExpr, RestPat};

use super::ExtraSpanInfo;
use super::ImportMap;
use crate::parser_plugin::{self, JavaScriptParserPluginDrive, JavascriptParserPlugin};
use crate::utils::eval::{self, BasicEvaluatedExpression};
use crate::visitors::scope_info::{
  FreeName, ScopeInfoDB, ScopeInfoId, TagInfo, VariableInfo, VariableInfoId,
};

pub trait TagInfoData: Clone {
  fn serialize(data: &Self) -> serde_json::Value;
  fn deserialize(value: serde_json::Value) -> Self;
}

#[derive(Debug)]
pub struct ExtractedMemberExpressionChainData {
  object: Expr,
  members: Vec<Atom>,
  member_ranges: Vec<Span>,
}

bitflags! {
  #[derive(Clone, Copy)]
  pub struct AllowedMemberTypes: u8 {
    const CallExpression = 1 << 0;
    const Expression = 1 << 1;
  }
}

#[derive(Debug)]
pub enum MemberExpressionInfo {
  Call(CallExpressionInfo),
  Expression(ExpressionExpressionInfo),
}

#[derive(Debug)]
pub struct CallExpressionInfo {
  pub call: CallExpr,
  pub callee_name: String,
  pub root_info: ExportedVariableInfo,
}

#[derive(Debug)]
pub struct ExpressionExpressionInfo {
  pub name: String,
  pub root_info: ExportedVariableInfo,
}

#[derive(Debug, Clone)]
pub enum ExportedVariableInfo {
  Name(String),
  VariableInfo(VariableInfoId),
}

fn object_and_members_to_name(
  object: impl AsRef<str>,
  members_reversed: &[impl AsRef<str>],
) -> String {
  let mut name = String::from(object.as_ref());
  let iter = members_reversed.iter();
  for member in iter.rev() {
    name.push('.');
    name.push_str(member.as_ref());
  }
  name
}

pub trait RootName {
  fn get_root_name(&self) -> Option<Atom> {
    None
  }
}

impl RootName for Expr {
  fn get_root_name(&self) -> Option<Atom> {
    match self {
      Expr::Ident(ident) => ident.get_root_name(),
      Expr::This(this) => this.get_root_name(),
      Expr::MetaProp(meta) => meta.get_root_name(),
      _ => None,
    }
  }
}

impl RootName for ThisExpr {
  fn get_root_name(&self) -> Option<Atom> {
    Some("this".into())
  }
}

impl RootName for Ident {
  fn get_root_name(&self) -> Option<Atom> {
    Some(self.sym.clone())
  }
}

impl RootName for MetaPropExpr {
  fn get_root_name(&self) -> Option<Atom> {
    match self.kind {
      MetaPropKind::NewTarget => Some("new.target".into()),
      MetaPropKind::ImportMeta => Some("import.meta".into()),
    }
  }
}

impl RootName for Callee {
  fn get_root_name(&self) -> Option<Atom> {
    match self {
      Callee::Expr(e) => e.get_root_name(),
      _ => None,
    }
  }
}

pub struct FreeInfo<'a> {
  pub name: &'a str,
  pub info: Option<&'a VariableInfo>,
}

#[derive(Clone, Copy, Debug)]
pub enum TopLevelScope {
  Top,
  ArrowFunction,
  False,
}

pub struct JavascriptParser<'parser> {
  pub(crate) source_file: &'parser SourceFile,
  pub(crate) errors: Vec<Box<dyn Diagnostic + Send + Sync>>,
  pub(crate) warning_diagnostics: Vec<Box<dyn Diagnostic + Send + Sync>>,
  pub(crate) dependencies: Vec<BoxDependency>,
  pub(crate) presentational_dependencies: Vec<Box<dyn DependencyTemplate>>,
  pub(crate) blocks: Vec<AsyncDependenciesBlock>,
  // TODO: remove `import_map`
  pub(crate) import_map: ImportMap,
  // TODO: remove `rewrite_usage_span`
  pub(crate) rewrite_usage_span: FxHashMap<Span, ExtraSpanInfo>,
  pub(crate) comments: Option<&'parser dyn Comments>,
  // TODO: remove `worker_syntax_list`
  pub(crate) worker_syntax_list: &'parser mut WorkerSyntaxList,
  pub(crate) worker_index: u32,
  pub(crate) build_meta: &'parser mut BuildMeta,
  pub(crate) build_info: &'parser mut BuildInfo,
  pub(crate) resource_data: &'parser ResourceData,
  pub(crate) plugin_drive: Rc<JavaScriptParserPluginDrive>,
  pub(crate) definitions_db: ScopeInfoDB,
  pub(crate) compiler_options: &'parser CompilerOptions,
  pub(crate) javascript_options: &'parser JavascriptParserOptions,
  pub(crate) module_type: &'parser ModuleType,
  pub(crate) module_identifier: &'parser ModuleIdentifier,
  // TODO: remove `is_esm` after `HarmonyExports::isEnabled`
  pub(crate) is_esm: bool,
  pub(crate) in_tagged_template_tag: bool,
  pub(crate) parser_exports_state: Option<bool>,
  // TODO: delete `enter_call`
  pub(crate) enter_call: u32,
  // TODO: delete `enter_new_expr`
  pub(crate) enter_new_expr: bool,
  // TODO: delete `enter_callee`
  pub(crate) enter_callee: bool,
  pub(crate) stmt_level: u32,
  pub(crate) last_stmt_is_expr_stmt: bool,
  // TODO: delete `properties_in_destructuring`
  pub(crate) properties_in_destructuring: FxHashMap<Atom, FxHashSet<Atom>>,
  // ===== scope info =======
  pub(crate) in_try: bool,
  pub(crate) in_short_hand: bool,
  pub(super) definitions: ScopeInfoId,
  pub(crate) top_level_scope: TopLevelScope,
  pub(crate) last_harmony_import_order: i32,
}

impl<'parser> JavascriptParser<'parser> {
  #[allow(clippy::too_many_arguments)]
  pub fn new(
    source_file: &'parser SourceFile,
    compiler_options: &'parser CompilerOptions,
    javascript_options: &'parser JavascriptParserOptions,
    comments: Option<&'parser dyn Comments>,
    module_identifier: &'parser ModuleIdentifier,
    module_type: &'parser ModuleType,
    worker_syntax_list: &'parser mut WorkerSyntaxList,
    resource_data: &'parser ResourceData,
    build_meta: &'parser mut BuildMeta,
    build_info: &'parser mut BuildInfo,
  ) -> Self {
    let warning_diagnostics: Vec<Box<dyn Diagnostic + Send + Sync>> = Vec::with_capacity(32);
    let errors = Vec::with_capacity(32);
    let dependencies = Vec::with_capacity(256);
    let blocks = Vec::with_capacity(256);
    let presentational_dependencies = Vec::with_capacity(256);
    let parser_exports_state: Option<bool> = None;
    let import_map = FxHashMap::default();
    let rewrite_usage_span = FxHashMap::default();

    let mut plugins: Vec<parser_plugin::BoxJavascriptParserPlugin> = Vec::with_capacity(32);
    plugins.push(Box::new(parser_plugin::InitializeEvaluating));
    plugins.push(Box::new(parser_plugin::JavascriptMetaInfoPlugin));
    plugins.push(Box::new(parser_plugin::CheckVarDeclaratorIdent));
    plugins.push(Box::new(parser_plugin::ConstPlugin));
    plugins.push(Box::new(
      parser_plugin::RequireContextDependencyParserPlugin,
    ));
    plugins.push(Box::new(parser_plugin::WorkerSyntaxScanner::new(
      rspack_core::needs_refactor::DEFAULT_WORKER_SYNTAX,
      worker_syntax_list,
    )));
    plugins.push(Box::new(parser_plugin::CompatibilityPlugin));

    if module_type.is_js_auto() || module_type.is_js_dynamic() {
      plugins.push(Box::new(parser_plugin::CommonJsImportsParserPlugin));
      plugins.push(Box::new(parser_plugin::CommonJsPlugin));
      plugins.push(Box::new(parser_plugin::CommonJsExportsParserPlugin));
      if compiler_options.node.is_some() {
        plugins.push(Box::new(parser_plugin::NodeStuffPlugin));
      }
    }

    if compiler_options.dev_server.hot {
      if module_type.is_js_auto() {
        plugins.push(Box::new(
          parser_plugin::hot_module_replacement::ModuleHotReplacementParserPlugin,
        ));
        plugins.push(Box::new(
          parser_plugin::hot_module_replacement::ImportMetaHotReplacementParserPlugin,
        ));
      } else if module_type.is_js_dynamic() {
        plugins.push(Box::new(
          parser_plugin::hot_module_replacement::ModuleHotReplacementParserPlugin,
        ));
      } else if module_type.is_js_esm() {
        plugins.push(Box::new(
          parser_plugin::hot_module_replacement::ImportMetaHotReplacementParserPlugin,
        ));
      }
    }

    if module_type.is_js_auto() || module_type.is_js_dynamic() || module_type.is_js_esm() {
      if !compiler_options.builtins.provide.is_empty() {
        plugins.push(Box::<parser_plugin::ProviderPlugin>::default());
      }
      plugins.push(Box::new(parser_plugin::WebpackIsIncludedPlugin));
      plugins.push(Box::new(parser_plugin::ExportsInfoApiPlugin));
      plugins.push(Box::new(parser_plugin::APIPlugin::new(
        compiler_options.output.module,
      )));
      plugins.push(Box::new(parser_plugin::ImportParserPlugin));
    }

    if module_type.is_js_auto() || module_type.is_js_esm() {
      let parse_url = javascript_options.url;
      if !matches!(parse_url, JavascriptParserUrl::Disable) {
        plugins.push(Box::new(parser_plugin::URLPlugin {
          relative: matches!(parse_url, JavascriptParserUrl::Relative),
        }));
      }
      plugins.push(Box::new(parser_plugin::HarmonyTopLevelThisParserPlugin));
      plugins.push(Box::new(parser_plugin::HarmonyDetectionParserPlugin::new(
        compiler_options.experiments.top_level_await,
      )));
      plugins.push(Box::new(parser_plugin::WorkerPlugin));
      plugins.push(Box::new(
        parser_plugin::ImportMetaContextDependencyParserPlugin,
      ));
      plugins.push(Box::new(parser_plugin::ImportMetaPlugin));
      plugins.push(Box::new(parser_plugin::HarmonyImportDependencyParserPlugin));
      plugins.push(Box::new(parser_plugin::HarmonyExportDependencyParserPlugin));
    }

    let plugin_drive = Rc::new(JavaScriptParserPluginDrive::new(plugins));
    let mut db = ScopeInfoDB::new();

    Self {
      last_harmony_import_order: 0,
      comments,
      javascript_options,
      source_file,
      errors,
      warning_diagnostics,
      dependencies,
      presentational_dependencies,
      blocks,
      in_try: false,
      in_short_hand: false,
      top_level_scope: TopLevelScope::Top,
      is_esm: matches!(module_type, ModuleType::JsEsm),
      in_tagged_template_tag: false,
      definitions: db.create(),
      definitions_db: db,
      plugin_drive,
      worker_syntax_list,
      resource_data,
      build_meta,
      build_info,
      compiler_options,
      module_type,
      parser_exports_state,
      enter_call: 0,
      stmt_level: 0,
      last_stmt_is_expr_stmt: false,
      worker_index: 0,
      module_identifier,
      import_map,
      rewrite_usage_span,
      enter_new_expr: false,
      enter_callee: false,
      properties_in_destructuring: Default::default(),
    }
  }

  pub fn get_mut_variable_info(&mut self, name: &str) -> Option<&mut VariableInfo> {
    let Some(id) = self.definitions_db.get(&self.definitions, name) else {
      return None;
    };
    Some(self.definitions_db.expect_get_mut_variable(&id))
  }

  pub fn get_variable_info(&mut self, name: &str) -> Option<&VariableInfo> {
    let Some(id) = self.definitions_db.get(&self.definitions, name) else {
      return None;
    };
    Some(self.definitions_db.expect_get_variable(&id))
  }

  pub fn get_free_info_from_variable<'a>(&'a mut self, name: &'a str) -> Option<FreeInfo<'a>> {
    let Some(info) = self.get_variable_info(name) else {
      return Some(FreeInfo { name, info: None });
    };
    let Some(FreeName::String(name)) = &info.free_name else {
      return None;
    };
    Some(FreeInfo {
      name,
      info: Some(info),
    })
  }

  pub fn get_all_variables_from_current_scope(
    &self,
  ) -> impl Iterator<Item = (&str, &VariableInfoId)> {
    let scope = self.definitions_db.expect_get_scope(&self.definitions);
    scope.variables()
  }

  fn define_variable(&mut self, name: String) {
    let definitions = self.definitions;
    if let Some(variable_info) = self.get_variable_info(&name)
      && variable_info.tag_info.is_some()
      && definitions == variable_info.declared_scope
    {
      return;
    }
    let info = VariableInfo::new(definitions, None, None);
    self.definitions_db.set(definitions, name, info);
  }

  fn set_variable(&mut self, name: String, variable: String) {
    let id = self.definitions;
    if name == variable {
      self.definitions_db.delete(id, &name);
    } else {
      let variable = VariableInfo::new(id, Some(FreeName::String(variable)), None);
      self.definitions_db.set(id, name, variable);
    }
  }

  fn undefined_variable(&mut self, name: String) {
    self.definitions_db.delete(self.definitions, name)
  }

  pub fn tag_variable<Data: TagInfoData>(
    &mut self,
    name: String,
    tag: &'static str,
    data: Option<Data>,
  ) {
    let data = data.as_ref().map(|data| TagInfoData::serialize(data));
    let new_info = if let Some(old_info_id) = self.definitions_db.get(&self.definitions, &name) {
      let old_info = self.definitions_db.expect_get_variable(&old_info_id);
      if let Some(old_tag_info) = &old_info.tag_info {
        // FIXME: remove `.clone`
        let free_name = old_info.free_name.clone();
        let tag_info = Some(TagInfo {
          tag,
          data,
          // FIXME: remove `.clone`
          next: Some(Box::new(old_tag_info.clone())),
        });
        VariableInfo::new(old_info.declared_scope, free_name, tag_info)
      } else {
        let free_name = Some(FreeName::True);
        let tag_info = Some(TagInfo {
          tag,
          data,
          next: None,
        });
        VariableInfo::new(old_info.declared_scope, free_name, tag_info)
      }
    } else {
      let free_name = Some(FreeName::String(name.clone()));
      let tag_info = Some(TagInfo {
        tag,
        data,
        next: None,
      });
      VariableInfo::new(self.definitions, free_name, tag_info)
    };
    self.definitions_db.set(self.definitions, name, new_info);
  }

  fn _get_member_expression_info(
    &mut self,
    object: Expr,
    members: Vec<Atom>,
    _members_range: Vec<Span>,
    allowed_types: AllowedMemberTypes,
  ) -> Option<MemberExpressionInfo> {
    match object {
      Expr::Call(expr) => {
        if !allowed_types.contains(AllowedMemberTypes::CallExpression) {
          return None;
        }
        let Some(root_name) = expr.callee.get_root_name() else {
          return None;
        };
        let Some(FreeInfo {
          name: resolved_root,
          info: root_info,
        }) = self.get_free_info_from_variable(&root_name)
        else {
          return None;
        };
        let callee_name = object_and_members_to_name(resolved_root, &members);
        Some(MemberExpressionInfo::Call(CallExpressionInfo {
          call: expr,
          callee_name,
          root_info: root_info
            .map(|i| ExportedVariableInfo::VariableInfo(i.id()))
            .unwrap_or_else(|| ExportedVariableInfo::Name(root_name.to_string())),
        }))
      }
      Expr::MetaProp(_) | Expr::Ident(_) | Expr::This(_) => {
        if !allowed_types.contains(AllowedMemberTypes::Expression) {
          return None;
        }
        let Some(root_name) = object.get_root_name() else {
          return None;
        };
        let Some(FreeInfo {
          name: resolved_root,
          info: root_info,
        }) = self.get_free_info_from_variable(&root_name)
        else {
          return None;
        };
        let name = object_and_members_to_name(resolved_root, &members);
        Some(MemberExpressionInfo::Expression(ExpressionExpressionInfo {
          name,
          root_info: root_info
            .map(|i| ExportedVariableInfo::VariableInfo(i.id()))
            .unwrap_or_else(|| ExportedVariableInfo::Name(root_name.to_string())),
        }))
      }
      _ => None,
    }
  }

  fn get_member_expression_info_from_expr(
    &mut self,
    expr: &Expr,
    allowed_types: AllowedMemberTypes,
  ) -> Option<MemberExpressionInfo> {
    expr
      .as_member()
      .and_then(|member| self.get_member_expression_info(member, allowed_types))
      .or_else(|| self._get_member_expression_info(expr.clone(), vec![], vec![], allowed_types))
  }

  fn get_member_expression_info(
    &mut self,
    expr: &MemberExpr,
    allowed_types: AllowedMemberTypes,
  ) -> Option<MemberExpressionInfo> {
    let ExtractedMemberExpressionChainData {
      object,
      members,
      member_ranges,
    } = Self::extract_member_expression_chain(expr);
    self._get_member_expression_info(object, members, member_ranges, allowed_types)
  }

  fn extract_member_expression_chain(expr: &MemberExpr) -> ExtractedMemberExpressionChainData {
    let mut object = Expr::Member(expr.clone());
    let mut members = Vec::new();
    let mut member_ranges = Vec::new();
    while let Some(expr) = object.as_mut_member() {
      if let Some(computed) = expr.prop.as_computed() {
        let Expr::Lit(lit) = &*computed.expr else {
          break;
        };
        let value = match lit {
          Lit::Str(s) => s.value.clone(),
          Lit::Bool(b) => if b.value { "true" } else { "false" }.into(),
          Lit::Null(_) => "null".into(),
          Lit::Num(n) => n.value.to_string().into(),
          Lit::BigInt(i) => i.value.to_string().into(),
          Lit::Regex(r) => r.exp.clone(),
          Lit::JSXText(_) => unreachable!(),
        };
        members.push(value);
        member_ranges.push(expr.obj.span());
      } else if let Some(ident) = expr.prop.as_ident() {
        members.push(ident.sym.clone());
        member_ranges.push(expr.obj.span());
      } else {
        break;
      }
      object = *expr.obj.take();
    }
    ExtractedMemberExpressionChainData {
      object,
      members,
      member_ranges,
    }
  }

  fn enter_ident<F>(&mut self, ident: &Ident, on_ident: F)
  where
    F: FnOnce(&mut Self, &Ident),
  {
    // TODO: add hooks here;
    on_ident(self, ident);
  }

  fn enter_array_pattern<F>(&mut self, array_pat: &ArrayPat, on_ident: F)
  where
    F: FnOnce(&mut Self, &Ident) + Copy,
  {
    array_pat
      .elems
      .iter()
      .flatten()
      .for_each(|ele| self.enter_pattern(Cow::Borrowed(ele), on_ident));
  }

  fn enter_assignment_pattern<F>(&mut self, assign: &AssignPat, on_ident: F)
  where
    F: FnOnce(&mut Self, &Ident) + Copy,
  {
    self.enter_pattern(Cow::Borrowed(&assign.left), on_ident);
  }

  fn enter_object_pattern<F>(&mut self, obj: &ObjectPat, on_ident: F)
  where
    F: FnOnce(&mut Self, &Ident) + Copy,
  {
    for prop in &obj.props {
      match prop {
        ObjectPatProp::KeyValue(kv) => self.enter_pattern(Cow::Borrowed(&kv.value), on_ident),
        ObjectPatProp::Assign(assign) => self.enter_ident(&assign.key, on_ident),
        ObjectPatProp::Rest(rest) => self.enter_rest_pattern(rest, on_ident),
      }
    }
  }

  fn enter_rest_pattern<F>(&mut self, rest: &RestPat, on_ident: F)
  where
    F: FnOnce(&mut Self, &Ident) + Copy,
  {
    self.enter_pattern(Cow::Borrowed(&rest.arg), on_ident)
  }

  fn enter_pattern<F>(&mut self, pattern: Cow<Pat>, on_ident: F)
  where
    F: FnOnce(&mut Self, &Ident) + Copy,
  {
    match &*pattern {
      Pat::Ident(ident) => self.enter_ident(&ident.id, on_ident),
      Pat::Array(array) => self.enter_array_pattern(array, on_ident),
      Pat::Assign(assign) => self.enter_assignment_pattern(assign, on_ident),
      Pat::Object(obj) => self.enter_object_pattern(obj, on_ident),
      Pat::Rest(rest) => self.enter_rest_pattern(rest, on_ident),
      Pat::Invalid(_) => (),
      Pat::Expr(_) => (),
    }
  }

  fn enter_assign_target_pattern<F>(&mut self, pattern: Cow<AssignTargetPat>, on_ident: F)
  where
    F: FnOnce(&mut Self, &Ident) + Copy,
  {
    match &*pattern {
      AssignTargetPat::Array(array) => self.enter_array_pattern(array, on_ident),
      AssignTargetPat::Object(obj) => self.enter_object_pattern(obj, on_ident),
      AssignTargetPat::Invalid(_) => (),
    }
  }

  fn enter_patterns<'a, I, F>(&mut self, patterns: I, on_ident: F)
  where
    F: FnOnce(&mut Self, &Ident) + Copy,
    I: Iterator<Item = Cow<'a, Pat>>,
  {
    for pattern in patterns {
      self.enter_pattern(pattern, on_ident);
    }
  }

  pub fn walk_program(&mut self, ast: &Program) {
    if self.plugin_drive.clone().program(self, ast).is_none() {
      match ast {
        Program::Module(m) => {
          self.set_strict(true);
          self.pre_walk_module_declarations(&m.body);
          self.block_pre_walk_module_declarations(&m.body);
          self.walk_module_declarations(&m.body);
        }
        Program::Script(s) => {
          self.detect_mode(&s.body);
          self.pre_walk_statements(&s.body);
          self.block_pre_walk_statements(&s.body);
          self.walk_statements(&s.body);
        }
      };
    }
    self.plugin_drive.clone().finish(self);
  }

  fn set_strict(&mut self, value: bool) {
    let current_scope = self.definitions_db.expect_get_mut_scope(&self.definitions);
    current_scope.is_strict = value;
  }

  fn detect_mode(&mut self, stmts: &[Stmt]) {
    let Some(Lit::Str(str)) = stmts
      .first()
      .and_then(|stmt| stmt.as_expr())
      .and_then(|expr_stmt| expr_stmt.expr.as_lit())
    else {
      return;
    };

    if str.value.as_str() == "use strict" {
      self.set_strict(true);
    }
  }

  pub fn is_strict(&mut self) -> bool {
    let scope = self.definitions_db.expect_get_scope(&self.definitions);
    scope.is_strict
  }

  // TODO: remove
  pub fn is_unresolved_ident(&mut self, str: &str) -> bool {
    self.definitions_db.get(&self.definitions, str).is_none()
  }
}

impl JavascriptParser<'_> {
  pub fn evaluate_expression(&mut self, expr: &Expr) -> BasicEvaluatedExpression {
    match self.evaluating(expr) {
      Some(evaluated) => evaluated,
      None => BasicEvaluatedExpression::with_range(expr.span().real_lo(), expr.span_hi().0),
    }
  }

  // same as `JavascriptParser._initializeEvaluating` in webpack
  // FIXME: should mv it to plugin(for example `parse.hooks.evaluate for`)
  fn evaluating(&mut self, expr: &Expr) -> Option<BasicEvaluatedExpression> {
    match expr {
      Expr::Tpl(tpl) => eval::eval_tpl_expression(self, tpl),
      Expr::TaggedTpl(tagged_tpl) => eval::eval_tagged_tpl_expression(self, tagged_tpl),
      Expr::Lit(lit) => eval::eval_lit_expr(lit),
      Expr::Cond(cond) => eval::eval_cond_expression(self, cond),
      Expr::Unary(unary) => eval::eval_unary_expression(self, unary),
      Expr::Bin(binary) => eval::eval_binary_expression(self, binary),
      Expr::Array(array) => eval::eval_array_expression(self, array),
      Expr::New(new) => eval::eval_new_expression(self, new),
      Expr::Call(call) => eval::eval_call_expression(self, call),
      Expr::Paren(paren) => self.evaluating(&paren.expr),
      Expr::Member(member) => {
        if let Some(MemberExpressionInfo::Expression(info)) =
          self.get_member_expression_info(member, AllowedMemberTypes::Expression)
        {
          self
            .plugin_drive
            .clone()
            .evaluate_identifier(self, &info.name, member.span.real_lo(), member.span.hi().0)
            .or_else(|| {
              // TODO: fallback with `evaluateDefinedIdentifier`
              let mut eval =
                BasicEvaluatedExpression::with_range(member.span.real_lo(), member.span.hi().0);
              eval.set_identifier(info.name, info.root_info);
              Some(eval)
            })
        } else {
          None
        }
      }
      Expr::Ident(ident) => {
        let drive = self.plugin_drive.clone();
        let Some(info) = self.get_variable_info(&ident.sym) else {
          // use `ident.sym` as fallback for global variable(or maybe just a undefined variable)
          return drive
            .evaluate_identifier(
              self,
              ident.sym.as_str(),
              ident.span.real_lo(),
              ident.span.hi.0,
            )
            .or_else(|| {
              let mut eval =
                BasicEvaluatedExpression::with_range(ident.span.real_lo(), ident.span.hi.0);

              if ident.sym.eq("undefined") {
                eval.set_undefined();
              } else {
                eval.set_identifier(
                  ident.sym.to_string(),
                  ExportedVariableInfo::Name(ident.sym.to_string()),
                );
              }

              Some(eval)
            });
        };
        if let Some(FreeName::String(name)) = info.free_name.as_ref() {
          // avoid ownership
          let name = name.to_string();
          return drive.evaluate_identifier(self, &name, ident.span.real_lo(), ident.span.hi.0);
        }
        None
      }
      Expr::This(this) => {
        let drive = self.plugin_drive.clone();
        let default_eval = || {
          let mut eval = BasicEvaluatedExpression::with_range(this.span.real_lo(), this.span.hi.0);
          eval.set_identifier(
            "this".to_string(),
            ExportedVariableInfo::Name("this".to_string()),
          );
          Some(eval)
        };
        let Some(info) = self.get_variable_info("this") else {
          // use `ident.sym` as fallback for global variable(or maybe just a undefined variable)
          return drive
            .evaluate_identifier(self, "this", this.span.real_lo(), this.span.hi.0)
            .or_else(default_eval);
        };
        if let Some(FreeName::String(name)) = info.free_name.as_ref() {
          // avoid ownership
          let name = name.to_string();
          return drive
            .evaluate_identifier(self, &name, this.span.real_lo(), this.span.hi.0)
            .or_else(default_eval);
        }
        None
      }
      _ => None,
    }
  }
}
