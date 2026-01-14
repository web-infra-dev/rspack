pub mod ast;
mod call_hooks_name;
pub mod estree;
mod walk;
mod walk_block_pre;
mod walk_module_pre;
mod walk_pre;

use std::{
  borrow::Cow,
  fmt::Display,
  hash::{Hash, Hasher},
  rc::Rc,
};

use bitflags::bitflags;
pub use call_hooks_name::CallHooksName;
use once_cell::unsync::OnceCell;
use ropey::Rope;
use rspack_cacheable::{
  cacheable,
  with::{AsCacheable, AsOption, AsPreset, AsVec},
};
use rspack_core::{
  AsyncDependenciesBlock, BoxDependency, BoxDependencyTemplate, BuildInfo, BuildMeta,
  CompilerOptions, DependencyRange, FactoryMeta, JavascriptParserCommonjsExportsOption,
  JavascriptParserOptions, ModuleIdentifier, ModuleLayer, ModuleType, ParseMeta, ResourceData,
  RuntimeTemplate, SideEffectsBailoutItemWithSpan,
};
use rspack_error::{Diagnostic, Result};
use rspack_util::{SpanExt, fx_hash::FxIndexSet};
use rustc_hash::{FxHashMap, FxHashSet};
use swc_core::{
  atoms::Atom,
  common::{BytePos, Mark, Span, Spanned, comments::Comments},
  ecma::{
    ast::{
      ArrayPat, AssignPat, AssignTargetPat, CallExpr, Decl, Expr, Ident, Lit, MemberExpr,
      MetaPropExpr, MetaPropKind, ObjectPat, ObjectPatProp, OptCall, OptChainBase, OptChainExpr,
      Pat, Program, RestPat, Stmt, ThisExpr,
    },
    utils::ExprFactory,
  },
};

use crate::{
  BoxJavascriptParserPlugin,
  dependency::local_module::LocalModule,
  parser_plugin::{
    self, ImportsReferencesState, InnerGraphState, JavaScriptParserPluginDrive,
    JavascriptParserPlugin,
  },
  utils::eval::{self, BasicEvaluatedExpression},
  visitors::{
    ScanDependenciesResult,
    dependency::parser::ast::ExprRef,
    scope_info::{
      ScopeInfoDB, ScopeInfoId, TagInfo, TagInfoId, VariableInfo, VariableInfoFlags, VariableInfoId,
    },
  },
};

pub trait TagInfoData: Clone + Sized + 'static {
  fn into_any(data: Self) -> Box<dyn anymap::CloneAny>;

  fn downcast(any: Box<dyn anymap::CloneAny>) -> Self;
}

impl<T> TagInfoData for T
where
  T: Clone + Sized + 'static,
{
  fn into_any(data: Self) -> Box<dyn anymap::CloneAny> {
    Box::new(data)
  }

  fn downcast(any: Box<dyn anymap::CloneAny>) -> Self {
    *(any as Box<dyn std::any::Any>)
      .downcast()
      .expect("TagInfoData should be downcasted from correct tag info")
  }
}

#[derive(Debug)]
pub struct ExtractedMemberExpressionChainData<'ast> {
  pub object: ExprRef<'ast>,
  pub members: Vec<Atom>,
  pub members_optionals: Vec<bool>,
  pub member_ranges: Vec<Span>,
}

bitflags! {
  #[derive(Clone, Copy)]
  pub struct AllowedMemberTypes: u8 {
    const CallExpression = 1 << 0;
    const Expression = 1 << 1;
  }
}

#[derive(Debug)]
pub enum MemberExpressionInfo<'ast> {
  Call(CallExpressionInfo<'ast>),
  Expression(ExpressionExpressionInfo),
}

#[derive(Debug)]
pub struct CallExpressionInfo<'ast> {
  pub call: &'ast CallExpr,
  pub root_info: ExportedVariableInfo,
  pub callee_members: Vec<Atom>,
  pub members: Vec<Atom>,
  pub members_optionals: Vec<bool>,
  pub member_ranges: Vec<Span>,
}

#[derive(Debug)]
pub struct ExpressionExpressionInfo {
  pub name: String,
  pub root_info: ExportedVariableInfo,
  pub members: Vec<Atom>,
  pub members_optionals: Vec<bool>,
  pub member_ranges: Vec<Span>,
}

#[derive(Debug, Clone)]
pub enum ExportedVariableInfo {
  Name(Atom),
  VariableInfo(VariableInfoId),
}

fn object_and_members_to_name(object: String, members_reversed: &[impl AsRef<str>]) -> String {
  let mut name = object;
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

impl RootName for ExprRef<'_> {
  fn get_root_name(&self) -> Option<Atom> {
    match self {
      ExprRef::Ident(ident) => ident.get_root_name(),
      ExprRef::This(this) => this.get_root_name(),
      ExprRef::MetaProp(meta) => meta.get_root_name(),
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

pub struct NameInfo<'a> {
  pub name: &'a Atom,
  pub info: Option<&'a VariableInfo>,
}

#[derive(Clone, Copy, Debug)]
pub enum TopLevelScope {
  Top,
  ArrowFunction,
  False,
}

#[derive(Debug, Clone, Copy)]
pub struct StatementPath {
  span: Span,
}

impl Spanned for StatementPath {
  fn span(&self) -> Span {
    self.span
  }
}

impl StatementPath {
  fn from_span(span: Span) -> Self {
    Self { span }
  }
}

impl From<Span> for StatementPath {
  fn from(value: Span) -> Self {
    Self::from_span(value)
  }
}

#[cacheable]
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct DestructuringAssignmentProperty {
  pub range: DependencyRange,
  #[cacheable(with=AsPreset)]
  pub id: Atom,
  #[cacheable(omit_bounds, with=AsOption<AsCacheable>)]
  pub pattern: Option<DestructuringAssignmentProperties>,
  pub shorthand: bool,
}

#[cacheable]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DestructuringAssignmentProperties {
  #[cacheable(with=AsVec<AsCacheable>)]
  inner: FxIndexSet<DestructuringAssignmentProperty>,
}

impl Hash for DestructuringAssignmentProperties {
  fn hash<H: Hasher>(&self, state: &mut H) {
    for prop in &self.inner {
      prop.hash(state);
    }
  }
}

impl DestructuringAssignmentProperties {
  pub fn new(properties: FxIndexSet<DestructuringAssignmentProperty>) -> Self {
    Self { inner: properties }
  }

  pub fn insert(&mut self, prop: DestructuringAssignmentProperty) -> bool {
    self.inner.insert(prop)
  }

  pub fn extend(&mut self, other: Self) {
    self.inner.extend(other.inner);
  }

  pub fn iter(&self) -> impl Iterator<Item = &DestructuringAssignmentProperty> {
    self.inner.iter()
  }

  pub fn traverse_on_leaf<'a, F>(&'a self, on_leaf_node: &mut F)
  where
    F: FnMut(&mut Vec<&'a DestructuringAssignmentProperty>),
  {
    self.traverse_impl(on_leaf_node, &mut |_| {}, &mut Vec::new());
  }

  pub fn traverse_on_enter<'a, F>(&'a self, on_enter_node: &mut F)
  where
    F: FnMut(&mut Vec<&'a DestructuringAssignmentProperty>),
  {
    self.traverse_impl(&mut |_| {}, on_enter_node, &mut Vec::new());
  }

  fn traverse_impl<'a, L, E>(
    &'a self,
    on_leaf_node: &mut L,
    on_enter_node: &mut E,
    stack: &mut Vec<&'a DestructuringAssignmentProperty>,
  ) where
    L: FnMut(&mut Vec<&'a DestructuringAssignmentProperty>),
    E: FnMut(&mut Vec<&'a DestructuringAssignmentProperty>),
  {
    for prop in &self.inner {
      stack.push(prop);
      on_enter_node(stack);
      if let Some(pattern) = &prop.pattern {
        pattern.traverse_impl(on_leaf_node, on_enter_node, stack);
      } else {
        on_leaf_node(stack);
      }
      stack.pop();
    }
  }
}

#[derive(Debug, Default)]
pub struct DestructuringAssignmentPropertiesMap {
  inner: FxHashMap<Span, DestructuringAssignmentProperties>,
}

impl DestructuringAssignmentPropertiesMap {
  pub fn add(&mut self, span: Span, props: DestructuringAssignmentProperties) {
    self.inner.entry(span).or_default().extend(props)
  }

  pub fn get(&self, span: &Span) -> Option<&DestructuringAssignmentProperties> {
    self.inner.get(span)
  }
}

pub struct JavascriptParser<'parser> {
  // ===== results =======
  errors: Vec<Diagnostic>,
  warning_diagnostics: Vec<Diagnostic>,
  dependencies: Vec<BoxDependency>,
  presentational_dependencies: Vec<BoxDependencyTemplate>,
  // Vec<Box<T: Sized>> makes sense if T is a large type (see #3530, 1st comment).
  // #3530: https://github.com/rust-lang/rust-clippy/issues/3530
  #[allow(clippy::vec_box)]
  blocks: Vec<Box<AsyncDependenciesBlock>>,
  // ===== inputs =======
  source_rope: OnceCell<Rope>,
  pub(crate) source: &'parser str,
  pub parse_meta: ParseMeta,
  pub comments: Option<&'parser dyn Comments>,
  pub factory_meta: Option<&'parser FactoryMeta>,
  pub build_meta: &'parser mut BuildMeta,
  pub build_info: &'parser mut BuildInfo,
  pub resource_data: &'parser ResourceData,
  pub(crate) compiler_options: &'parser CompilerOptions,
  pub(crate) javascript_options: &'parser JavascriptParserOptions,
  pub runtime_template: &'parser RuntimeTemplate,
  pub module_type: &'parser ModuleType,
  pub(crate) module_layer: Option<&'parser ModuleLayer>,
  pub module_identifier: &'parser ModuleIdentifier,
  pub(crate) plugin_drive: Rc<JavaScriptParserPluginDrive>,
  // ===== states =======
  pub(crate) definitions_db: ScopeInfoDB,
  pub(super) definitions: ScopeInfoId,
  pub(crate) top_level_scope: TopLevelScope,
  pub(crate) current_tag_info: Option<TagInfoId>,
  pub in_try: bool,
  pub(crate) in_short_hand: bool,
  pub(crate) in_tagged_template_tag: bool,
  pub(crate) member_expr_in_optional_chain: bool,
  pub(crate) semicolons: &'parser mut FxHashSet<BytePos>,
  pub(crate) statement_path: Vec<StatementPath>,
  pub(crate) prev_statement: Option<StatementPath>,
  pub is_esm: bool,
  pub(crate) destructuring_assignment_properties: DestructuringAssignmentPropertiesMap,
  pub(crate) dynamic_import_references: ImportsReferencesState,
  pub(crate) worker_index: u32,
  pub(crate) parser_exports_state: Option<bool>,
  pub(crate) local_modules: Vec<LocalModule>,
  pub(crate) last_esm_import_order: i32,
  pub(crate) inner_graph: InnerGraphState,
  pub(crate) has_inlinable_const_decls: bool,
  pub(crate) side_effects_item: Option<SideEffectsBailoutItemWithSpan>,
}

impl<'parser> JavascriptParser<'parser> {
  #[allow(clippy::too_many_arguments)]
  pub fn new(
    source: &'parser str,
    compiler_options: &'parser CompilerOptions,
    javascript_options: &'parser JavascriptParserOptions,
    comments: Option<&'parser dyn Comments>,
    module_identifier: &'parser ModuleIdentifier,
    module_type: &'parser ModuleType,
    module_layer: Option<&'parser ModuleLayer>,
    resource_data: &'parser ResourceData,
    factory_meta: Option<&'parser FactoryMeta>,
    build_meta: &'parser mut BuildMeta,
    build_info: &'parser mut BuildInfo,
    semicolons: &'parser mut FxHashSet<BytePos>,
    unresolved_mark: Mark,
    parser_plugins: &'parser mut Vec<BoxJavascriptParserPlugin>,
    parse_meta: ParseMeta,
    runtime_template: &'parser RuntimeTemplate,
  ) -> Self {
    let warning_diagnostics: Vec<Diagnostic> = Vec::with_capacity(4);
    let errors = Vec::with_capacity(4);
    let dependencies = Vec::with_capacity(64);
    let blocks = Vec::with_capacity(64);
    let presentational_dependencies = Vec::with_capacity(64);
    let parser_exports_state: Option<bool> = None;

    let mut plugins: Vec<BoxJavascriptParserPlugin> = Vec::with_capacity(32);

    plugins.append(parser_plugins);

    plugins.push(Box::new(parser_plugin::InitializeEvaluating));
    plugins.push(Box::new(parser_plugin::JavascriptMetaInfoPlugin));
    plugins.push(Box::new(parser_plugin::CheckVarDeclaratorIdent));
    plugins.push(Box::new(parser_plugin::ConstPlugin));
    plugins.push(Box::new(parser_plugin::UseStrictPlugin));
    plugins.push(Box::new(
      parser_plugin::RequireContextDependencyParserPlugin,
    ));
    plugins.push(Box::new(
      parser_plugin::RequireEnsureDependenciesBlockParserPlugin,
    ));
    plugins.push(Box::new(parser_plugin::CompatibilityPlugin));

    if module_type.is_js_auto() || module_type.is_js_esm() {
      plugins.push(Box::new(parser_plugin::ESMTopLevelThisParserPlugin));
      plugins.push(Box::new(parser_plugin::ESMDetectionParserPlugin::default()));
      plugins.push(Box::new(
        parser_plugin::ImportMetaContextDependencyParserPlugin,
      ));
      if let Some(true) = javascript_options.import_meta {
        plugins.push(Box::new(parser_plugin::ImportMetaPlugin));
      } else {
        plugins.push(Box::new(parser_plugin::ImportMetaDisabledPlugin));
      }

      plugins.push(Box::new(parser_plugin::ESMImportDependencyParserPlugin));
      plugins.push(Box::new(parser_plugin::ESMExportDependencyParserPlugin));
    }

    if compiler_options.amd.is_some() && (module_type.is_js_auto() || module_type.is_js_dynamic()) {
      plugins.push(Box::new(
        parser_plugin::AMDRequireDependenciesBlockParserPlugin,
      ));
      plugins.push(Box::new(parser_plugin::AMDDefineDependencyParserPlugin));
      plugins.push(Box::new(parser_plugin::AMDParserPlugin));
    }

    if module_type.is_js_auto() || module_type.is_js_dynamic() {
      plugins.push(Box::new(parser_plugin::CommonJsImportsParserPlugin));
      plugins.push(Box::new(parser_plugin::CommonJsPlugin));
      let commonjs_exports = javascript_options
        .commonjs
        .as_ref()
        .map_or(JavascriptParserCommonjsExportsOption::Enable, |commonjs| {
          commonjs.exports
        });
      if commonjs_exports != JavascriptParserCommonjsExportsOption::Disable {
        plugins.push(Box::new(parser_plugin::CommonJsExportsParserPlugin::new(
          commonjs_exports == JavascriptParserCommonjsExportsOption::SkipInEsm,
        )));
      }
    }

    // NodeStuffPlugin: handle __dirname/__filename/global (CJS) and import.meta.dirname/filename (ESM)
    // CJS features require node options; ESM features are always available for ESM-capable modules
    let handle_cjs =
      (module_type.is_js_auto() || module_type.is_js_dynamic()) && compiler_options.node.is_some();
    let handle_esm = module_type.is_js_auto() || module_type.is_js_esm();
    if handle_cjs || handle_esm {
      plugins.push(Box::new(parser_plugin::NodeStuffPlugin::new(
        handle_cjs, handle_esm,
      )));
    }

    if module_type.is_js_auto() || module_type.is_js_dynamic() || module_type.is_js_esm() {
      plugins.push(Box::new(parser_plugin::IsIncludedPlugin));
      plugins.push(Box::new(parser_plugin::ExportsInfoApiPlugin));
      plugins.push(Box::new(parser_plugin::APIPlugin::new(
        compiler_options.output.module,
      )));
      plugins.push(Box::new(parser_plugin::ImportParserPlugin));
      plugins.push(Box::new(parser_plugin::WorkerPlugin::new(
        javascript_options
          .worker
          .as_ref()
          .expect("should have worker"),
      )));
      plugins.push(Box::new(parser_plugin::OverrideStrictPlugin));
    }

    if compiler_options.optimization.inline_exports {
      build_info.inline_exports = true;
      plugins.push(Box::new(parser_plugin::InlineConstPlugin));
    }
    if compiler_options.optimization.inner_graph {
      plugins.push(Box::new(parser_plugin::InnerGraphPlugin::new(
        unresolved_mark,
      )));
    }

    if compiler_options.optimization.side_effects.is_true() {
      plugins.push(Box::new(parser_plugin::SideEffectsParserPlugin::new(
        unresolved_mark,
      )));
    }

    let plugin_drive = Rc::new(JavaScriptParserPluginDrive::new(plugins));
    let mut db = ScopeInfoDB::new();

    Self {
      last_esm_import_order: 0,
      comments,
      javascript_options,
      source_rope: OnceCell::new(),
      source,
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
      resource_data,
      factory_meta,
      build_meta,
      build_info,
      compiler_options,
      module_type,
      module_layer,
      parser_exports_state,
      worker_index: 0,
      module_identifier,
      member_expr_in_optional_chain: false,
      destructuring_assignment_properties: Default::default(),
      dynamic_import_references: Default::default(),
      semicolons,
      statement_path: Default::default(),
      current_tag_info: None,
      prev_statement: None,
      inner_graph: InnerGraphState::new(),
      parse_meta,
      local_modules: Default::default(),
      has_inlinable_const_decls: true,
      side_effects_item: None,
      runtime_template,
    }
  }

  pub fn into_results(self) -> Result<ScanDependenciesResult, Vec<Diagnostic>> {
    if self.errors.is_empty() {
      Ok(ScanDependenciesResult {
        dependencies: self.dependencies,
        blocks: self.blocks,
        presentational_dependencies: self.presentational_dependencies,
        warning_diagnostics: self.warning_diagnostics,
        side_effects_item: self.side_effects_item,
      })
    } else {
      Err(self.errors)
    }
  }

  pub fn add_dependency(&mut self, dep: BoxDependency) {
    self.dependencies.push(dep);
  }

  pub fn add_dependencies(&mut self, deps: impl IntoIterator<Item = BoxDependency>) {
    self.dependencies.extend(deps);
  }

  pub fn pop_dependency(&mut self) -> Option<BoxDependency> {
    self.dependencies.pop()
  }

  pub fn next_dependency_idx(&self) -> usize {
    self.dependencies.len()
  }

  pub fn get_dependencies(&self) -> &[BoxDependency] {
    &self.dependencies
  }

  pub fn get_dependency_mut(&mut self, idx: usize) -> Option<&mut BoxDependency> {
    self.dependencies.get_mut(idx)
  }

  pub fn collect_dependencies_for_block(
    &mut self,
    f: impl FnOnce(&mut JavascriptParser),
  ) -> Vec<BoxDependency> {
    let old_deps = std::mem::take(&mut self.dependencies);
    f(self);
    std::mem::replace(&mut self.dependencies, old_deps)
  }

  pub fn add_presentational_dependency(&mut self, dep: BoxDependencyTemplate) {
    self.presentational_dependencies.push(dep);
  }

  pub fn add_presentational_dependencies(
    &mut self,
    deps: impl IntoIterator<Item = BoxDependencyTemplate>,
  ) {
    self.presentational_dependencies.extend(deps);
  }

  pub fn next_presentational_dependency_idx(&self) -> usize {
    self.presentational_dependencies.len()
  }

  pub fn get_presentational_dependency_mut(
    &mut self,
    idx: usize,
  ) -> Option<&mut BoxDependencyTemplate> {
    self.presentational_dependencies.get_mut(idx)
  }

  pub fn add_block(&mut self, block: Box<AsyncDependenciesBlock>) {
    self.blocks.push(block);
  }

  pub fn next_block_idx(&self) -> usize {
    self.blocks.len()
  }

  pub fn get_block_mut(&mut self, idx: usize) -> Option<&mut Box<AsyncDependenciesBlock>> {
    self.blocks.get_mut(idx)
  }

  pub fn add_error(&mut self, error: Diagnostic) {
    self.errors.push(error);
  }

  pub fn add_warning(&mut self, warning: Diagnostic) {
    self.warning_diagnostics.push(warning);
  }

  pub fn add_warnings(&mut self, warnings: impl IntoIterator<Item = Diagnostic>) {
    self.warning_diagnostics.extend(warnings);
  }

  pub fn source(&self) -> &str {
    self.source
  }

  pub fn source_rope(&mut self) -> &Rope {
    self.source_rope.get_or_init(|| Rope::from_str(self.source))
  }

  pub fn is_top_level_scope(&self) -> bool {
    matches!(self.top_level_scope, TopLevelScope::Top)
  }

  pub fn is_top_level_this(&self) -> bool {
    !matches!(self.top_level_scope, TopLevelScope::False)
  }

  pub fn add_local_module(&mut self, name: &Atom, dep_idx: usize) {
    self.local_modules.push(LocalModule::new(
      name.clone(),
      self.local_modules.len(),
      dep_idx,
    ));
  }

  pub fn get_local_module_mut(&mut self, name: &str) -> Option<&mut LocalModule> {
    self.local_modules.iter_mut().find(|m| m.get_name() == name)
  }

  pub fn is_asi_position(&self, pos: BytePos) -> bool {
    let curr_path = self.statement_path.last().expect("Should in statement");
    if curr_path.span_hi() == pos && self.semicolons.contains(&pos) {
      true
    } else if curr_path.span_lo() == pos
      && let Some(prev) = &self.prev_statement
      && self.semicolons.contains(&prev.span_hi())
    {
      true
    } else {
      false
    }
  }

  pub fn set_asi_position(&mut self, pos: BytePos) -> bool {
    self.semicolons.insert(pos)
  }

  pub fn unset_asi_position(&mut self, pos: BytePos) -> bool {
    self.semicolons.remove(&pos)
  }

  pub fn is_statement_level_expression(&self, expr_span: Span) -> bool {
    let Some(curr_path) = self.statement_path.last() else {
      return false;
    };
    curr_path.span() == expr_span
  }

  pub fn get_module_layer(&self) -> Option<&ModuleLayer> {
    self.module_layer
  }

  pub fn get_variable_info(&mut self, name: &Atom) -> Option<&VariableInfo> {
    let id = self.definitions_db.get(self.definitions, name)?;
    Some(self.definitions_db.expect_get_variable(id))
  }

  pub fn get_tag_data(
    &mut self,
    name: &Atom,
    tag: &'static str,
  ) -> Option<Box<dyn anymap::CloneAny>> {
    self
      .get_variable_info(name)
      .and_then(|variable_info| variable_info.tag_info)
      .and_then(|tag_info_id| {
        let mut tag_info = Some(self.definitions_db.expect_get_tag_info(tag_info_id));

        while let Some(cur_tag_info) = tag_info {
          if cur_tag_info.tag == tag {
            return cur_tag_info.data.clone();
          }
          tag_info = cur_tag_info
            .next
            .map(|tag_info_id| self.definitions_db.expect_get_tag_info(tag_info_id))
        }

        None
      })
  }

  pub fn get_free_info_from_variable<'a>(&'a mut self, name: &'a Atom) -> Option<NameInfo<'a>> {
    let Some(info) = self.get_variable_info(name) else {
      return Some(NameInfo { name, info: None });
    };
    let Some(name) = &info.name else {
      return None;
    };
    if !info.is_free() {
      return None;
    }
    Some(NameInfo {
      name,
      info: Some(info),
    })
  }

  pub fn get_name_info_from_variable<'a>(&'a mut self, name: &'a Atom) -> Option<NameInfo<'a>> {
    let Some(info) = self.get_variable_info(name) else {
      return Some(NameInfo { name, info: None });
    };
    let Some(name) = &info.name else {
      return None;
    };
    if !info.is_free() && !info.is_tagged() {
      return None;
    }
    Some(NameInfo {
      name,
      info: Some(info),
    })
  }

  pub fn get_all_variables_from_current_scope(
    &self,
  ) -> impl Iterator<Item = (&str, &VariableInfoId)> {
    let scope = self.definitions_db.expect_get_scope(self.definitions);
    scope.variables()
  }

  pub fn define_variable(&mut self, name: Atom) {
    let definitions = self.definitions;
    if let Some(variable_info) = self.get_variable_info(&name)
      && variable_info.tag_info.is_some()
      && definitions == variable_info.declared_scope
    {
      return;
    }
    let info = VariableInfo::create(
      &mut self.definitions_db,
      definitions,
      None,
      VariableInfoFlags::NORMAL,
      None,
    );
    self.definitions_db.set(definitions, name, info);
  }

  pub fn set_variable(&mut self, name: Atom, variable: ExportedVariableInfo) {
    let scope_id = self.definitions;
    match variable {
      ExportedVariableInfo::Name(variable) => {
        if name == variable {
          self.definitions_db.delete(scope_id, &name);
        } else {
          let variable = VariableInfo::create(
            &mut self.definitions_db,
            scope_id,
            Some(variable),
            VariableInfoFlags::FREE,
            None,
          );
          self.definitions_db.set(scope_id, name, variable);
        }
      }
      ExportedVariableInfo::VariableInfo(variable) => {
        self.definitions_db.set(scope_id, name, variable);
      }
    }
  }

  fn undefined_variable(&mut self, name: &Atom) {
    self.definitions_db.delete(self.definitions, name)
  }

  pub fn tag_variable<Data: TagInfoData>(
    &mut self,
    name: Atom,
    tag: &'static str,
    data: Option<Data>,
  ) {
    self.tag_variable_impl(name, tag, data, None);
  }

  pub fn tag_variable_with_flags<Data: TagInfoData>(
    &mut self,
    name: Atom,
    tag: &'static str,
    data: Option<Data>,
    flags: VariableInfoFlags,
  ) {
    self.tag_variable_impl(name, tag, data, Some(flags));
  }

  fn tag_variable_impl<Data: TagInfoData>(
    &mut self,
    name: Atom,
    tag: &'static str,
    data: Option<Data>,
    flags: Option<VariableInfoFlags>,
  ) {
    let flags = flags.unwrap_or(VariableInfoFlags::TAGGED);
    let data = data.map(|data| TagInfoData::into_any(data));
    let new_info = if let Some(old_info_id) = self.definitions_db.get(self.definitions, &name) {
      let old_info = self.definitions_db.expect_get_variable(old_info_id);
      if let Some(old_tag_info) = old_info.tag_info {
        let declared_scope = old_info.declared_scope;
        // FIXME: remove `.clone`
        let name = old_info.name.clone();
        let flags = old_info.flags | flags;
        let tag_info = Some(TagInfo::create(
          &mut self.definitions_db,
          tag,
          data,
          Some(old_tag_info),
        ));
        VariableInfo::create(
          &mut self.definitions_db,
          declared_scope,
          name,
          flags,
          tag_info,
        )
      } else {
        let declared_scope = old_info.declared_scope;
        let tag_info = Some(TagInfo::create(&mut self.definitions_db, tag, data, None));
        VariableInfo::create(
          &mut self.definitions_db,
          declared_scope,
          Some(name.clone()),
          flags,
          tag_info,
        )
      }
    } else {
      let tag_info = Some(TagInfo::create(&mut self.definitions_db, tag, data, None));
      VariableInfo::create(
        &mut self.definitions_db,
        self.definitions,
        Some(name.clone()),
        flags,
        tag_info,
      )
    };
    self.definitions_db.set(self.definitions, name, new_info);
  }

  fn _get_member_expression_info<'ast>(
    &mut self,
    object: ExprRef<'ast>,
    mut members: Vec<Atom>,
    mut members_optionals: Vec<bool>,
    mut member_ranges: Vec<Span>,
    allowed_types: AllowedMemberTypes,
  ) -> Option<MemberExpressionInfo<'ast>> {
    match object {
      ExprRef::Call(expr) => {
        if !allowed_types.contains(AllowedMemberTypes::CallExpression) {
          return None;
        }
        let callee = expr.callee.as_expr()?;
        let (root_name, mut root_members) = if let Some(member) = callee.as_member() {
          let extracted = self.extract_member_expression_chain(ExprRef::Member(member));
          let root_name = extracted.object.get_root_name()?;
          (root_name, extracted.members)
        } else {
          (callee.get_root_name()?, vec![])
        };
        let NameInfo {
          info: root_info, ..
        } = self.get_name_info_from_variable(&root_name)?;

        root_members.reverse();
        members.reverse();
        members_optionals.reverse();
        member_ranges.reverse();
        Some(MemberExpressionInfo::Call(CallExpressionInfo {
          call: expr,
          root_info: root_info
            .map(|i| ExportedVariableInfo::VariableInfo(i.id()))
            .unwrap_or_else(|| ExportedVariableInfo::Name(root_name)),
          callee_members: root_members,
          members,
          members_optionals,
          member_ranges,
        }))
      }
      ExprRef::MetaProp(_) | ExprRef::Ident(_) | ExprRef::This(_) => {
        if !allowed_types.contains(AllowedMemberTypes::Expression) {
          return None;
        }
        let root_name = object.get_root_name()?;

        let NameInfo {
          name: resolved_root,
          info: root_info,
        } = self.get_name_info_from_variable(&root_name)?;

        let name = object_and_members_to_name(resolved_root.to_string(), &members);
        members.reverse();
        members_optionals.reverse();
        member_ranges.reverse();
        Some(MemberExpressionInfo::Expression(ExpressionExpressionInfo {
          name,
          root_info: root_info
            .map(|i| ExportedVariableInfo::VariableInfo(i.id()))
            .unwrap_or_else(|| ExportedVariableInfo::Name(root_name)),
          members,
          members_optionals,
          member_ranges,
        }))
      }
      _ => None,
    }
  }

  pub fn get_member_expression_info_from_expr<'ast>(
    &mut self,
    expr: &'ast Expr,
    allowed_types: AllowedMemberTypes,
  ) -> Option<MemberExpressionInfo<'ast>> {
    match expr {
      Expr::Member(_) | Expr::OptChain(_) => {
        self.get_member_expression_info(expr.into(), allowed_types)
      }
      _ => self._get_member_expression_info(expr.into(), vec![], vec![], vec![], allowed_types),
    }
  }

  pub fn get_member_expression_info<'ast>(
    &mut self,
    expr: ExprRef<'ast>,
    allowed_types: AllowedMemberTypes,
  ) -> Option<MemberExpressionInfo<'ast>> {
    let ExtractedMemberExpressionChainData {
      object,
      members,
      members_optionals,
      member_ranges,
    } = self.extract_member_expression_chain(expr);
    self._get_member_expression_info(
      object,
      members,
      members_optionals,
      member_ranges,
      allowed_types,
    )
  }

  pub fn extract_member_expression_chain<'ast>(
    &self,
    expr: ExprRef<'ast>,
  ) -> ExtractedMemberExpressionChainData<'ast> {
    let mut object = expr;
    let mut members = Vec::new();
    let mut members_optionals = Vec::new();
    let mut member_ranges = Vec::new();
    let mut in_optional_chain = self.member_expr_in_optional_chain;
    loop {
      match object {
        ExprRef::Member(expr) => {
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
              Lit::Regex(r) => r.exp.clone().into(),
              Lit::JSXText(_) => unreachable!(),
            };
            // Since members are not used across rspack javascript parser plugin,
            // we directly makes it atom here
            members.push(value.to_atom_lossy().into_owned());
            member_ranges.push(expr.obj.span());
          } else if let Some(ident) = expr.prop.as_ident() {
            members.push(ident.sym.clone());
            member_ranges.push(expr.obj.span());
          } else {
            break;
          }
          members_optionals.push(in_optional_chain);
          object = expr.obj.as_ref().into();
          in_optional_chain = false;
        }
        ExprRef::OptChain(expr) => {
          in_optional_chain = expr.optional;
          if let OptChainBase::Member(member) = expr.base.as_ref() {
            object = ExprRef::Member(member);
          } else {
            break;
          }
        }
        _ => break,
      }
    }
    ExtractedMemberExpressionChainData {
      object,
      members,
      members_optionals,
      member_ranges,
    }
  }

  fn enter_ident<F>(&mut self, ident: &Ident, on_ident: F)
  where
    F: FnOnce(&mut Self, &Ident),
  {
    if !ident
      .sym
      .call_hooks_name(self, |parser, for_name| {
        parser.plugin_drive.clone().pattern(parser, ident, for_name)
      })
      .unwrap_or_default()
    {
      on_ident(self, ident);
    }
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
        ObjectPatProp::Assign(assign) => {
          let old = self.in_short_hand;
          if assign.value.is_none() {
            self.in_short_hand = true;
          }
          self.enter_ident(&assign.key, on_ident);
          self.in_short_hand = old;
        }
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

  fn enter_optional_chain<'a, C, M, R>(
    &mut self,
    expr: &'a OptChainExpr,
    on_call: C,
    on_member: M,
  ) -> R
  where
    C: FnOnce(&mut Self, &'a OptCall) -> R,
    M: FnOnce(&mut Self, &'a MemberExpr) -> R,
  {
    let member_expr_in_optional_chain = self.member_expr_in_optional_chain;
    let ret = match &*expr.base {
      OptChainBase::Call(call) => {
        if call.callee.is_member() {
          self.member_expr_in_optional_chain = expr.optional;
        }
        on_call(self, call)
      }
      OptChainBase::Member(member) => {
        self.member_expr_in_optional_chain = expr.optional;
        on_member(self, member)
      }
    };
    self.member_expr_in_optional_chain = member_expr_in_optional_chain;
    ret
  }

  fn enter_declaration<F>(&mut self, decl: &Decl, on_ident: F)
  where
    F: FnOnce(&mut Self, &Ident) + Copy,
  {
    match decl {
      Decl::Class(c) => {
        self.enter_ident(&c.ident, on_ident);
      }
      Decl::Fn(f) => {
        self.enter_ident(&f.ident, on_ident);
      }
      Decl::Var(var) => {
        for decl in &var.decls {
          self.enter_pattern(Cow::Borrowed(&decl.name), on_ident);
        }
      }
      Decl::Using(_) => (),
      _ => unreachable!(),
    }
  }

  fn enter_statement<S, H, F>(&mut self, statement: &S, call_hook: H, on_statement: F)
  where
    S: Spanned,
    H: FnOnce(&mut Self, &S) -> bool,
    F: FnOnce(&mut Self, &S),
  {
    self.statement_path.push(statement.span().into());
    if call_hook(self, statement) {
      self.prev_statement = self.statement_path.pop();
      return;
    }
    on_statement(self, statement);
    self.prev_statement = self.statement_path.pop();
  }

  pub fn enter_destructuring_assignment<'a>(
    &mut self,
    pattern: &ObjectPat,
    expr: &'a Expr,
  ) -> Option<&'a Expr> {
    let expr = if let Some(await_expr) = expr.as_await_expr() {
      &await_expr.arg
    } else {
      expr
    };
    let destructuring = if let Some(assign) = expr.as_assign()
      && let Some(pat) = assign.left.as_pat()
      && let Some(obj_pat) = pat.as_object()
    {
      self.enter_destructuring_assignment(obj_pat, &assign.right)
    } else {
      let can_collect = self
        .plugin_drive
        .clone()
        .can_collect_destructuring_assignment_properties(self, expr)
        .unwrap_or_default();
      can_collect.then_some(expr)
    };
    if let Some(destructuring) = destructuring
      && let Some(keys) =
        self.collect_destructuring_assignment_properties_from_object_pattern(pattern)
    {
      self
        .destructuring_assignment_properties
        .add(destructuring.span(), keys);
    }
    destructuring
  }

  pub fn walk_program(&mut self, ast: &Program) {
    if self.plugin_drive.clone().program(self, ast).is_none() {
      match ast {
        Program::Module(m) => {
          self.set_strict(true);
          self.prev_statement = None;
          self.module_pre_walk_module_items(&m.body);
          self.prev_statement = None;
          self.pre_walk_module_items(&m.body);
          self.prev_statement = None;
          self.block_pre_walk_module_items(&m.body);
          self.prev_statement = None;
          self.walk_module_items(&m.body);
        }
        Program::Script(s) => {
          self.detect_mode(&s.body);
          self.prev_statement = None;
          self.pre_walk_statements(&s.body);
          self.prev_statement = None;
          self.block_pre_walk_statements(&s.body);
          self.prev_statement = None;
          self.walk_statements(&s.body);
        }
      };
    }
    self.plugin_drive.clone().finish(self);
  }

  fn set_strict(&mut self, value: bool) {
    let current_scope = self.definitions_db.expect_get_mut_scope(self.definitions);
    current_scope.is_strict = value;
  }

  pub fn detect_mode(&mut self, stmts: &[Stmt]) {
    let Some(Lit::Str(str)) = stmts
      .first()
      .and_then(|stmt| stmt.as_expr())
      .and_then(|expr_stmt| expr_stmt.expr.as_lit())
    else {
      return;
    };

    if str.value == "use strict" {
      self.set_strict(true);
    }
  }

  pub fn is_strict(&mut self) -> bool {
    let scope = self.definitions_db.expect_get_scope(self.definitions);
    scope.is_strict
  }

  pub fn is_variable_defined(&mut self, name: &Atom) -> bool {
    let Some(info) = self.get_variable_info(name) else {
      return false;
    };
    !info.is_free()
  }
}

impl JavascriptParser<'_> {
  pub fn evaluate_expression<'a>(&mut self, expr: &'a Expr) -> BasicEvaluatedExpression<'a> {
    match self.evaluating(expr) {
      Some(evaluated) => evaluated.with_expression(Some(expr)),
      None => BasicEvaluatedExpression::with_range(expr.span().real_lo(), expr.span().real_hi())
        .with_expression(Some(expr)),
    }
  }

  pub fn evaluate<T: Display>(
    &mut self,
    source: String,
    error_title: T,
  ) -> Option<BasicEvaluatedExpression<'static>> {
    eval::eval_source(self, source, error_title)
  }

  // same as `JavascriptParser._initializeEvaluating` in webpack
  // FIXME: should mv it to plugin(for example `parse.hooks.evaluate for`)
  fn evaluating<'a>(&mut self, expr: &'a Expr) -> Option<BasicEvaluatedExpression<'a>> {
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
      Expr::OptChain(opt_chain) => self.enter_optional_chain(
        opt_chain,
        |parser, call| {
          let expr = Expr::Call(CallExpr {
            ctxt: call.ctxt,
            span: call.span,
            callee: call.callee.clone().as_callee(),
            args: call.args.clone(),
            type_args: None,
          });
          BasicEvaluatedExpression::with_owned_expression(expr, |expr| {
            #[allow(clippy::unwrap_used)]
            let call_expr = expr.as_call().unwrap();
            eval::eval_call_expression(parser, call_expr)
          })
        },
        |parser, member| eval::eval_member_expression(parser, member, expr),
      ),
      Expr::Member(member) => eval::eval_member_expression(self, member, expr),
      Expr::Ident(ident) => {
        let name = &ident.sym;
        if name.eq("undefined") {
          let mut eval =
            BasicEvaluatedExpression::with_range(ident.span.real_lo(), ident.span.real_hi());
          eval.set_undefined();
          return Some(eval);
        }
        let drive = self.plugin_drive.clone();
        name
          .call_hooks_name(self, |parser, name| {
            drive.evaluate_identifier(parser, name, ident.span.real_lo(), ident.span.real_hi())
          })
          .or_else(|| {
            let info = self.get_variable_info(name);
            if let Some(info) = info {
              if let Some(name) = &info.name
                && (info.is_free() || info.is_tagged())
              {
                let mut eval =
                  BasicEvaluatedExpression::with_range(ident.span.real_lo(), ident.span.real_hi());
                eval.set_identifier(
                  name.to_owned(),
                  ExportedVariableInfo::VariableInfo(info.id()),
                  None,
                  None,
                  None,
                );
                Some(eval)
              } else {
                None
              }
            } else {
              let mut eval =
                BasicEvaluatedExpression::with_range(ident.span.real_lo(), ident.span.real_hi());
              eval.set_identifier(
                ident.sym.clone(),
                ExportedVariableInfo::Name(name.clone()),
                None,
                None,
                None,
              );
              Some(eval)
            }
          })
      }
      Expr::This(this) => {
        let drive = self.plugin_drive.clone();
        let default_eval = || {
          let mut eval =
            BasicEvaluatedExpression::with_range(this.span.real_lo(), this.span.real_hi());
          eval.set_identifier(
            "this".into(),
            ExportedVariableInfo::Name("this".into()),
            None,
            None,
            None,
          );
          Some(eval)
        };
        let Some(info) = self.get_variable_info(&"this".into()) else {
          // use `ident.sym` as fallback for global variable(or maybe just a undefined variable)
          return drive
            .evaluate_identifier(self, "this", this.span.real_lo(), this.span.real_hi())
            .or_else(default_eval);
        };
        if let Some(name) = &info.name
          && (info.is_free() || info.is_tagged())
        {
          let name = name.clone();
          return drive
            .evaluate_identifier(self, &name, this.span.real_lo(), this.span.real_hi())
            .or_else(default_eval);
        }
        None
      }
      _ => None,
    }
  }
}
