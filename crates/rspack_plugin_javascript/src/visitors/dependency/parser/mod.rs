use rspack_util::SpanExt;
pub mod ast;
mod call_hooks_name;
pub mod estree;
mod location_advancer;
mod walk;
mod walk_block_pre;
mod walk_module_pre;
mod walk_pre;

use std::{
  fmt::Display,
  hash::{Hash, Hasher},
  rc::Rc,
  sync::Arc,
};

use bitflags::bitflags;
pub use call_hooks_name::CallHooksName;
use rspack_cacheable::{
  cacheable,
  with::{AsCacheable, AsOption, AsPreset, AsVec},
};
use rspack_core::{
  ArcComputed, AsyncDependenciesBlock, BoxDependency, BuildInfo, BuildMeta, CompilerOptions,
  DependencyCodeGeneration, DependencyCodeGenerationRef, DependencyId, DependencyLocation,
  DependencyRange, FactoryMeta, ImportMeta, ImportMetaKnownProperties,
  JavascriptParserCommonjsExportsOption, JavascriptParserOptions, ModuleIdentifier, ModuleLayer,
  ModuleType, ParseMeta, ResolvedModuleOptions, ResourceData, SideEffectsBailoutItemWithSpan,
};
use rspack_error::{Diagnostic, Result};
use rspack_util::fx_hash::FxIndexSet;
use rustc_hash::{FxHashMap, FxHashSet};
use smallvec::SmallVec;
use swc_next_ecma_ast::{
  ArrayPattern, AssignmentPattern, Ast, BindingIdentifier, BindingPattern, BindingPatternData,
  BindingRestElement, CallExpression, Decl, DeclData, Expr, ExprData, GetSpan, IdentifierReference,
  MemberExpression, MetaProperty, ObjectPattern, Program, PropertyKey, PropertyKeyData, Span,
  StmtData, ThisExpression,
};

use crate::{
  Atom, BoxJavascriptParserPlugin,
  dependency::{DependencyBranchGuard, local_module::LocalModule},
  parser_and_generator::ParserRuntimeRequirementsData,
  parser_plugin::{
    self, CreatedRequireReferencesState, ImportsReferencesState, InnerGraphParserPlugin,
    JavaScriptParserPluginDrive, JavascriptParserPlugin, RequireReferencesState,
    inner_graph::state::InnerGraphState,
  },
  utils::eval::{self, BasicEvaluatedExpression},
  visitors::{
    ParsedJavaScriptAst, ScanDependenciesResult,
    dependency::parser::{ast::ExprRef, location_advancer::DependencyLocationAdvancer},
    scope_info::{
      ScopeInfoDB, ScopeInfoId, TagInfo, TagInfoId, VariableInfo, VariableInfoFlags, VariableInfoId,
    },
  },
};

pub trait TagInfoData: Clone + Sized + 'static {
  fn into_any(data: Self) -> Box<dyn anymap::CloneAny>;

  fn downcast(any: Box<dyn anymap::CloneAny>) -> Self;

  fn downcast_ref(any: &dyn anymap::CloneAny) -> &Self;

  fn downcast_mut(any: &mut dyn anymap::CloneAny) -> &mut Self;
}

fn atom_from_wtf8(value: &swc_next_allocator::wtf8::Wtf8) -> Atom {
  Atom::from(value.to_string_lossy().as_ref())
}

pub(crate) fn member_property_to_atom(ast: &Ast<'_>, expr: Expr) -> Option<Atom> {
  Some(match ast.expr_data(expr) {
    ExprData::StringLiteral(node) => atom_from_wtf8(ast.get_wtf8(node.value(ast))),
    ExprData::BooleanLiteral(node) => Atom::from(if node.value(ast) { "true" } else { "false" }),
    ExprData::NullLiteral(_) => Atom::from("null"),
    ExprData::NumericLiteral(node) => {
      Atom::from(rspack_util::ryu_js::Buffer::new().format(node.value(ast)))
    }
    ExprData::BigIntLiteral(node) => Atom::from(ast.get_utf8(node.raw(ast))),
    ExprData::RegExpLiteral(node) => {
      let pattern = ast.get_utf8(node.pattern(ast));
      let mut flags = ast.get_utf8(node.flags(ast)).chars().collect::<Vec<_>>();
      flags.sort_unstable();
      let mut property = String::with_capacity(pattern.len() + flags.len() + 2);
      property.push('/');
      property.push_str(pattern);
      property.push('/');
      property.extend(flags);
      Atom::from(property)
    }
    ExprData::TemplateLiteral(node)
      if node.expressions(ast).is_empty() && node.quasis(ast).len() == 1 =>
    {
      let quasi = node.quasis(ast).get_node(ast, 0)?;
      if quasi.is_cooked_undefined(ast) {
        Atom::from(ast.get_utf8(quasi.raw(ast)))
      } else {
        atom_from_wtf8(ast.get_wtf8(quasi.cooked(ast)))
      }
    }
    _ => return None,
  })
}

/// Resolve the key of a computed member expression.
///
/// `PropertyKey` is a nested union in SWC Next. Literal expressions such as
/// `obj["key"]` are exposed directly as `StringLiteral`, without an outer
/// `PropertyKeyData::Expr` variant.
pub(crate) fn member_property_key_to_atom(ast: &Ast<'_>, key: PropertyKey) -> Option<Atom> {
  member_property_key_data_to_atom(ast, ast.property_key_data(key))
}

fn member_property_key_data_to_atom(ast: &Ast<'_>, key: PropertyKeyData) -> Option<Atom> {
  match key {
    PropertyKeyData::StringLiteral(node) => Some(atom_from_wtf8(ast.get_wtf8(node.value(ast)))),
    PropertyKeyData::NumericLiteral(node) => Some(Atom::from(
      rspack_util::ryu_js::Buffer::new().format(node.value(ast)),
    )),
    PropertyKeyData::BigIntLiteral(node) => Some(Atom::from(ast.get_utf8(node.raw(ast)))),
    PropertyKeyData::Expr(expr) => member_property_to_atom(ast, expr),
    PropertyKeyData::IdentifierName(_) | PropertyKeyData::PrivateIdentifier(_) => None,
  }
}

fn member_property_key_data_can_be_atom(ast: &Ast<'_>, key: PropertyKeyData) -> bool {
  match key {
    PropertyKeyData::StringLiteral(_)
    | PropertyKeyData::NumericLiteral(_)
    | PropertyKeyData::BigIntLiteral(_) => true,
    PropertyKeyData::Expr(expr) => match ast.expr_data(expr) {
      ExprData::StringLiteral(_)
      | ExprData::BooleanLiteral(_)
      | ExprData::NullLiteral(_)
      | ExprData::NumericLiteral(_)
      | ExprData::BigIntLiteral(_)
      | ExprData::RegExpLiteral(_) => true,
      ExprData::TemplateLiteral(template) => {
        template.expressions(ast).is_empty() && template.quasis(ast).len() == 1
      }
      _ => false,
    },
    PropertyKeyData::IdentifierName(_) | PropertyKeyData::PrivateIdentifier(_) => false,
  }
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

  fn downcast_ref(any: &dyn anymap::CloneAny) -> &Self {
    let any = any as &dyn std::any::Any;
    any
      .downcast_ref()
      .expect("TagInfoData should be downcasted from correct tag info")
  }

  fn downcast_mut(any: &mut dyn anymap::CloneAny) -> &mut Self {
    let any = any as &mut dyn std::any::Any;
    any
      .downcast_mut()
      .expect("TagInfoData should be downcasted from correct tag info")
  }
}

// Most parsed member chains are one or two segments long, so keep them inline.
pub type AtomMembers = SmallVec<[Atom; 2]>;
pub type OptionalMembers = SmallVec<[bool; 2]>;
pub type MemberRanges = SmallVec<[Span; 2]>;
type RawAtomMembers = SmallVec<[PropertyKeyData; 2]>;

struct RawExtractedMemberExpressionChainData {
  object: ExprRef,
  members: RawAtomMembers,
  members_optionals: OptionalMembers,
  member_ranges: MemberRanges,
}

fn materialize_member_atoms(ast: &Ast<'_>, members: RawAtomMembers) -> AtomMembers {
  let mut atoms = AtomMembers::with_capacity(members.len());
  for property in members {
    atoms.push(match property {
      PropertyKeyData::IdentifierName(identifier) => Atom::from_ast(ast, identifier.name(ast)),
      property => member_property_key_data_to_atom(ast, property)
        .expect("validated computed member property should convert to an atom"),
    });
  }
  atoms
}

#[derive(Debug)]
pub struct ExtractedMemberExpressionChainData {
  pub object: ExprRef,
  pub members: AtomMembers,
  pub members_optionals: OptionalMembers,
  pub member_ranges: MemberRanges,
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
  pub call: CallExpression,
  pub root_info: ExportedVariableInfo,
  pub callee_members: AtomMembers,
  pub members: AtomMembers,
  pub members_optionals: OptionalMembers,
  pub member_ranges: MemberRanges,
}

#[derive(Debug)]
pub struct ExpressionExpressionInfo {
  pub name: String,
  pub root_info: ExportedVariableInfo,
  pub members: AtomMembers,
  pub members_optionals: OptionalMembers,
  pub member_ranges: MemberRanges,
}

#[derive(Debug, Clone)]
pub enum ExportedVariableInfo {
  Name(Atom),
  VariableInfo(VariableInfoId),
}

fn object_and_members_to_name(object: &str, members_reversed: &[impl AsRef<str>]) -> String {
  let total_len = object.len()
    + members_reversed.len()
    + members_reversed
      .iter()
      .map(|m| m.as_ref().len())
      .sum::<usize>();

  let mut name = String::with_capacity(total_len);
  name.push_str(object);
  let iter = members_reversed.iter();
  for member in iter.rev() {
    name.push('.');
    name.push_str(member.as_ref());
  }
  name
}

pub trait RootName {
  fn get_root_name<'ast>(&self, _ast: &'ast Ast<'_>) -> Option<&'ast str> {
    None
  }
}

impl RootName for Expr {
  #[inline]
  fn get_root_name<'ast>(&self, ast: &'ast Ast<'_>) -> Option<&'ast str> {
    match ast.expr_data(*self) {
      ExprData::IdentifierReference(ident) => ident.get_root_name(ast),
      ExprData::ThisExpression(this) => this.get_root_name(ast),
      ExprData::MetaProperty(meta) => meta.get_root_name(ast),
      _ => None,
    }
  }
}

impl RootName for ExprRef {
  #[inline]
  fn get_root_name<'ast>(&self, ast: &'ast Ast<'_>) -> Option<&'ast str> {
    match self {
      ExprRef::Ident(ident) => ident.get_root_name(ast),
      ExprRef::This(this) => this.get_root_name(ast),
      ExprRef::MetaProp(meta) => meta.get_root_name(ast),
      _ => None,
    }
  }
}

impl RootName for ThisExpression {
  #[inline]
  fn get_root_name<'ast>(&self, _ast: &'ast Ast<'_>) -> Option<&'ast str> {
    Some("this")
  }
}

impl RootName for IdentifierReference {
  #[inline]
  fn get_root_name<'ast>(&self, ast: &'ast Ast<'_>) -> Option<&'ast str> {
    Some(ast.get_utf8(self.name(ast)))
  }
}

impl RootName for MetaProperty {
  #[inline]
  fn get_root_name<'ast>(&self, ast: &'ast Ast<'_>) -> Option<&'ast str> {
    match (
      ast.get_utf8(self.meta(ast).name(ast)),
      ast.get_utf8(self.property(ast).name(ast)),
    ) {
      ("new", "target") => Some("new.target"),
      ("import", "meta") => Some("import.meta"),
      _ => None,
    }
  }
}

pub struct NameInfo<'a> {
  pub name: &'a str,
  pub info: Option<&'a VariableInfo>,
}

pub enum PatRef {
  Borrowed(BindingPattern),
  Owned(BindingPattern),
}

impl PatRef {
  pub(crate) fn as_pat(&self) -> BindingPattern {
    match self {
      PatRef::Borrowed(pattern) | PatRef::Owned(pattern) => *pattern,
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeTerminated {
  Return,
  Throw,
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

impl StatementPath {
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
      // Empty nested patterns still access and coerce their parent value, so
      // the parent property is a referenced leaf in that case.
      if let Some(pattern) = &prop.pattern
        && !pattern.inner.is_empty()
      {
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
  presentational_dependencies: Vec<DependencyCodeGenerationRef>,
  // Vec<Box<T: Sized>> makes sense if T is a large type (see #3530, 1st comment).
  // #3530: https://github.com/rust-lang/rust-clippy/issues/3530
  #[allow(clippy::vec_box)]
  blocks: Vec<Box<AsyncDependenciesBlock>>,
  // ===== inputs =======
  pub(crate) source: &'parser str,
  pub ast: &'parser ParsedJavaScriptAst<'parser>,
  pub parse_meta: ParseMeta,
  pub factory_meta: Option<&'parser FactoryMeta>,
  pub build_meta: &'parser mut BuildMeta,
  pub build_info: &'parser mut BuildInfo,
  pub resource_data: &'parser ResourceData,
  pub(crate) compiler_options: &'parser CompilerOptions,
  pub(crate) javascript_options: &'parser JavascriptParserOptions,
  pub parser_runtime_requirements: &'parser ParserRuntimeRequirementsData,
  pub module_type: &'parser ModuleType,
  pub(crate) module_layer: Option<&'parser ModuleLayer>,
  pub module_identifier: &'parser ModuleIdentifier,
  pub(crate) plugin_drive: Rc<JavaScriptParserPluginDrive>,
  // ===== states =======
  pub(crate) definitions_db: ScopeInfoDB,
  pub(crate) definitions: ScopeInfoId,
  pub(crate) top_level_scope: TopLevelScope,
  pub(crate) current_tag_info: Option<TagInfoId>,
  pub in_try: bool,
  pub(crate) terminated: Option<ScopeTerminated>,
  pub(crate) in_short_hand: bool,
  pub(crate) in_tagged_template_tag: bool,
  pub(crate) member_expr_in_optional_chain: bool,
  pub(crate) semicolons: &'parser mut FxHashSet<u32>,
  pub(crate) statement_path: Vec<StatementPath>,
  pub(crate) prev_statement: Option<StatementPath>,
  pub is_esm: bool,
  pub(crate) destructuring_assignment_properties: DestructuringAssignmentPropertiesMap,
  pub(crate) dynamic_import_references: ImportsReferencesState,
  pub(crate) common_js_require_references: RequireReferencesState,
  pub(crate) created_require_references: CreatedRequireReferencesState,
  pub(crate) worker_index: u32,
  pub(crate) parser_exports_state: Option<bool>,
  pub(crate) local_modules: Vec<LocalModule>,
  pub(crate) last_esm_import_order: i32,
  pub(crate) inner_graph: InnerGraphState,
  pub(crate) side_effects_item: Option<SideEffectsBailoutItemWithSpan>,
  pub(crate) is_renaming: Option<Atom>,
  pub(crate) location_advancer: DependencyLocationAdvancer,
  pub(crate) collecting_dependencies_for_block: Option<usize>,
  pub(crate) dependencies_in_branch_guard: Option<FxHashMap<DependencyRange, DependencyId>>,
  pub(crate) current_branch_guard: Option<DependencyBranchGuard>,
}

impl<'parser> JavascriptParser<'parser> {
  #[allow(clippy::too_many_arguments)]
  pub fn new(
    source: &'parser str,
    ast: &'parser ParsedJavaScriptAst<'parser>,
    compiler_options: &'parser CompilerOptions,
    javascript_options: &'parser JavascriptParserOptions,
    import_meta: ArcComputed<ResolvedModuleOptions, ImportMeta>,
    module_identifier: &'parser ModuleIdentifier,
    module_type: &'parser ModuleType,
    module_layer: Option<&'parser ModuleLayer>,
    resource_data: &'parser ResourceData,
    factory_meta: Option<&'parser FactoryMeta>,
    build_meta: &'parser mut BuildMeta,
    build_info: &'parser mut BuildInfo,
    semicolons: &'parser mut FxHashSet<u32>,
    parser_plugins: &'parser mut Vec<BoxJavascriptParserPlugin>,
    parse_meta: ParseMeta,
    parser_runtime_requirements: &'parser ParserRuntimeRequirementsData,
  ) -> Self {
    let warning_diagnostics: Vec<Diagnostic> = Vec::new();
    let errors = Vec::new();
    let dependencies = Vec::with_capacity(64);
    let blocks = Vec::with_capacity(64);
    let presentational_dependencies = Vec::with_capacity(64);
    let parser_exports_state: Option<bool> = None;

    let mut plugins: Vec<BoxJavascriptParserPlugin> = Vec::with_capacity(32 + parser_plugins.len());

    plugins.append(parser_plugins);

    plugins.push(Box::new(parser_plugin::InitializeEvaluating));
    plugins.push(Box::new(parser_plugin::JavascriptMetaInfoPlugin));
    plugins.push(Box::new(parser_plugin::ConstPlugin));
    plugins.push(Box::new(parser_plugin::UseStrictPlugin));

    if matches!(module_type, ModuleType::JsAuto | ModuleType::JsDynamic) {
      plugins.push(Box::new(
        parser_plugin::RequireContextDependencyParserPlugin,
      ));
      plugins.push(Box::new(
        parser_plugin::RequireEnsureDependenciesBlockParserPlugin,
      ));
    }
    plugins.push(Box::new(parser_plugin::CompatibilityPlugin));

    if module_type.is_js_auto() || module_type.is_js_esm() {
      plugins.push(Box::new(parser_plugin::ESMTopLevelThisParserPlugin));
      plugins.push(Box::<parser_plugin::ESMDetectionParserPlugin>::default());
      plugins.push(Box::new(
        parser_plugin::ImportMetaContextDependencyParserPlugin {
          webpack_context: import_meta
            .is_known_property_enabled(ImportMetaKnownProperties::WEBPACK_CONTEXT),
          glob: import_meta.is_known_property_enabled(ImportMetaKnownProperties::GLOB),
        },
      ));
      if import_meta.is_enabled() {
        plugins.push(Box::new(parser_plugin::ImportMetaPlugin(import_meta)));
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

    if module_type.is_js_auto() || module_type.is_js_dynamic() || module_type.is_js_esm() {
      plugins.push(Box::new(parser_plugin::CommonJsImportsParserPlugin));
    }

    if module_type.is_js_auto() || module_type.is_js_dynamic() {
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

    // NodeStuffPlugin handles __dirname/__filename/global (CJS).
    let handle_cjs =
      (module_type.is_js_auto() || module_type.is_js_dynamic()) && compiler_options.node.is_some();
    if handle_cjs {
      plugins.push(Box::new(parser_plugin::NodeStuffPlugin::new(handle_cjs)));
    }

    if module_type.is_js_auto() || module_type.is_js_dynamic() || module_type.is_js_esm() {
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

    let inline_exports = compiler_options.optimization.inline_exports;
    if inline_exports {
      build_info.inline_exports = true;
    }
    plugins.push(Box::new(parser_plugin::ConstValuePlugin::new(
      inline_exports,
    )));
    if compiler_options.optimization.inner_graph {
      plugins.push(Box::new(parser_plugin::InnerGraphParserPlugin::new(
        compiler_options.experiments.pure_functions,
      )));
    }

    if compiler_options.optimization.side_effects.is_true() {
      plugins.push(Box::new(parser_plugin::SideEffectsParserPlugin::new(
        compiler_options.experiments.pure_functions,
      )));
    }

    let plugin_drive = Rc::new(JavaScriptParserPluginDrive::new(plugins));
    let mut db = ScopeInfoDB::new();

    Self {
      last_esm_import_order: 0,
      ast,
      javascript_options,
      source,
      errors,
      warning_diagnostics,
      dependencies,
      presentational_dependencies,
      blocks,
      in_try: false,
      terminated: None,
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
      common_js_require_references: Default::default(),
      created_require_references: Default::default(),
      semicolons,
      statement_path: Default::default(),
      current_tag_info: None,
      prev_statement: None,
      inner_graph: InnerGraphState::new(),
      parse_meta,
      local_modules: Default::default(),
      side_effects_item: None,
      parser_runtime_requirements,
      is_renaming: None,
      location_advancer: DependencyLocationAdvancer::new(),
      collecting_dependencies_for_block: None,
      dependencies_in_branch_guard: None,
      current_branch_guard: None,
    }
  }

  pub fn into_results(mut self) -> Result<ScanDependenciesResult, Vec<Diagnostic>> {
    if self.errors.is_empty() {
      InnerGraphParserPlugin::finalize_dependency_usage(
        &mut self.inner_graph,
        &mut self.dependencies,
      );
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

  pub fn add_dependency(&mut self, mut dep: BoxDependency) {
    if let Some(guard) = &self.current_branch_guard {
      guard.bind_dependency(dep.as_mut());
    }
    self.dependencies.push(dep);
  }

  pub fn add_dependencies(&mut self, deps: impl IntoIterator<Item = BoxDependency>) {
    if let Some(guard) = &self.current_branch_guard {
      self.dependencies.extend(deps.into_iter().map(|mut dep| {
        guard.bind_dependency(dep.as_mut());
        dep
      }));
    } else {
      self.dependencies.extend(deps);
    }
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
    block_idx: usize,
    deps: Vec<BoxDependency>,
    f: impl FnOnce(&mut JavascriptParser),
  ) -> Vec<BoxDependency> {
    let old_deps = std::mem::replace(&mut self.dependencies, deps);
    let old_block_idx = self.collecting_dependencies_for_block.replace(block_idx);
    f(self);
    self.collecting_dependencies_for_block = old_block_idx;
    std::mem::replace(&mut self.dependencies, old_deps)
  }

  pub fn collect_dependencies_in_branch_guard<T>(
    &mut self,
    f: impl FnOnce(&mut JavascriptParser) -> T,
  ) -> T {
    let old_deps = self
      .dependencies_in_branch_guard
      .replace(Default::default());
    let result = f(self);
    self.dependencies_in_branch_guard = old_deps;
    result
  }

  pub fn with_branch_guard(&mut self, guard: DependencyBranchGuard, f: impl FnOnce(&mut Self)) {
    let guard = if let Some(old_guard) = &self.current_branch_guard {
      // handle for: if (A) { if (B) { import("./x") } }
      old_guard.clone().and(guard)
    } else {
      guard
    };
    let old_guard = self.current_branch_guard.replace(guard);
    f(self);
    self.current_branch_guard = old_guard;
  }

  pub fn add_presentational_dependency(&mut self, dep: DependencyCodeGenerationRef) {
    self.presentational_dependencies.push(dep);
  }

  pub fn add_presentational_dependencies(
    &mut self,
    deps: impl IntoIterator<Item = DependencyCodeGenerationRef>,
  ) {
    self.presentational_dependencies.extend(deps);
  }

  pub fn next_presentational_dependency_idx(&self) -> usize {
    self.presentational_dependencies.len()
  }

  pub fn get_presentational_dependency_mut(
    &mut self,
    idx: usize,
  ) -> Option<&mut (dyn DependencyCodeGeneration + 'static)> {
    Arc::get_mut(self.presentational_dependencies.get_mut(idx)?)
  }

  pub fn add_block(&mut self, mut block: Box<AsyncDependenciesBlock>) {
    if let Some(guard) = &self.current_branch_guard {
      for dep in block.dependencies_mut() {
        guard.bind_dependency(dep.as_mut());
      }
    }
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

  pub fn is_asi_position(&self, pos: u32) -> bool {
    let curr_path = self.statement_path.last().expect("Should in statement");
    if curr_path.span().end == pos && self.semicolons.contains(&pos) {
      true
    } else if curr_path.span().start == pos
      && let Some(prev) = &self.prev_statement
      && self.semicolons.contains(&prev.span().end)
    {
      true
    } else {
      false
    }
  }

  pub fn set_asi_position(&mut self, pos: u32) -> bool {
    self.semicolons.insert(pos)
  }

  pub fn unset_asi_position(&mut self, pos: u32) -> bool {
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

  /// The source order assigned to the import declaration currently being
  /// visited by `import_specifier` parser hooks.
  pub fn current_esm_import_order(&self) -> i32 {
    self.last_esm_import_order
  }

  pub fn get_variable_info(&mut self, name: &str) -> Option<&VariableInfo> {
    let id = self.definitions_db.get(self.definitions, name)?;
    Some(self.definitions_db.expect_get_variable(id))
  }

  fn get_tag_data_by_id<Data: TagInfoData>(
    &self,
    tag_info_id: TagInfoId,
    tag: &'static str,
  ) -> Option<&Data> {
    let mut cur = Some(tag_info_id);

    while let Some(cur_id) = cur {
      let cur_tag_info = self.definitions_db.expect_get_tag_info(cur_id);
      if cur_tag_info.tag == tag {
        return cur_tag_info
          .data
          .as_deref()
          .map(|data| TagInfoData::downcast_ref(data));
      }
      cur = cur_tag_info.next;
    }

    None
  }

  fn get_tag_data_mut_by_id<Data: TagInfoData>(
    &mut self,
    tag_info_id: TagInfoId,
    tag: &'static str,
  ) -> Option<&mut Data> {
    let mut cur = Some(tag_info_id);

    while let Some(cur_id) = cur {
      let cur_tag_info = self.definitions_db.expect_get_tag_info(cur_id);
      if cur_tag_info.tag == tag {
        return self
          .definitions_db
          .expect_get_mut_tag_info(cur_id)
          .data
          .as_deref_mut()
          .map(|data| TagInfoData::downcast_mut(data));
      }
      cur = cur_tag_info.next;
    }

    None
  }

  pub fn get_tag_data<Data: TagInfoData>(
    &mut self,
    name: &str,
    tag: &'static str,
  ) -> Option<&Data> {
    self
      .get_variable_info(name)
      .and_then(|variable_info| variable_info.tag_info)
      .and_then(|tag_info_id| self.get_tag_data_by_id(tag_info_id, tag))
  }

  pub fn get_tag_data_mut<Data: TagInfoData>(
    &mut self,
    name: &str,
    tag: &'static str,
  ) -> Option<&mut Data> {
    self
      .get_variable_info(name)
      .and_then(|variable_info| variable_info.tag_info)
      .and_then(|tag_info_id| self.get_tag_data_mut_by_id(tag_info_id, tag))
  }

  pub fn get_variable_tag_data<Data: TagInfoData>(
    &self,
    id: VariableInfoId,
    tag: &'static str,
  ) -> Option<&Data> {
    self
      .definitions_db
      .expect_get_variable(id)
      .tag_info
      .and_then(|tag_info_id| self.get_tag_data_by_id(tag_info_id, tag))
  }

  pub fn get_free_info_from_variable<'a>(&'a mut self, name: &'a str) -> Option<NameInfo<'a>> {
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
      name: name.as_str(),
      info: Some(info),
    })
  }

  pub fn get_name_info_from_variable<'a>(&'a mut self, name: &'a str) -> Option<NameInfo<'a>> {
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
      name: name.as_str(),
      info: Some(info),
    })
  }

  pub fn get_all_variables_from_current_scope(
    &self,
  ) -> impl Iterator<Item = (&Atom, VariableInfoId)> {
    self.definitions_db.scope_variables(self.definitions)
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
    self.tag_variable_impl(name, tag, data.map(TagInfoData::into_any), None);
  }

  pub fn tag_variable_with_flags<Data: TagInfoData>(
    &mut self,
    name: Atom,
    tag: &'static str,
    data: Option<Data>,
    flags: VariableInfoFlags,
  ) {
    self.tag_variable_impl(name, tag, data.map(TagInfoData::into_any), Some(flags));
  }

  pub fn tag_variable_without_data(&mut self, name: Atom, tag: &'static str) {
    self.tag_variable_impl(name, tag, None, None);
  }

  fn tag_variable_impl(
    &mut self,
    name: Atom,
    tag: &'static str,
    data: Option<Box<dyn anymap::CloneAny>>,
    flags: Option<VariableInfoFlags>,
  ) {
    let flags = flags.unwrap_or(VariableInfoFlags::TAGGED);
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

  fn _get_member_expression_info(
    &mut self,
    object: ExprRef,
    members: RawAtomMembers,
    mut members_optionals: OptionalMembers,
    mut member_ranges: MemberRanges,
    allowed_types: AllowedMemberTypes,
  ) -> Option<MemberExpressionInfo> {
    let ast = self.ast.ast;
    match object {
      ExprRef::Call(expr) => {
        if !allowed_types.contains(AllowedMemberTypes::CallExpression) {
          return None;
        }
        let callee = expr.callee(ast);
        let (root_name, root_members) = if let Some(member) = callee.as_member_expression(ast) {
          let extracted = self.extract_member_expression_chain_raw(ExprRef::Member(member));
          let root_name = extracted.object.get_root_name(ast)?;
          (root_name, extracted.members)
        } else {
          (callee.get_root_name(ast)?, RawAtomMembers::new())
        };
        let NameInfo {
          info: root_info, ..
        } = self.get_name_info_from_variable(root_name)?;

        let mut root_members = materialize_member_atoms(ast, root_members);
        let mut members = materialize_member_atoms(ast, members);
        root_members.reverse();
        members.reverse();
        members_optionals.reverse();
        member_ranges.reverse();
        Some(MemberExpressionInfo::Call(CallExpressionInfo {
          call: expr,
          root_info: root_info.map_or_else(
            || ExportedVariableInfo::Name(Atom::from(root_name)),
            |i| ExportedVariableInfo::VariableInfo(i.id()),
          ),
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
        let root_name = object.get_root_name(ast)?;

        let NameInfo {
          name: resolved_root,
          info: root_info,
        } = self.get_name_info_from_variable(root_name)?;

        let mut members = materialize_member_atoms(ast, members);
        let name = object_and_members_to_name(resolved_root, &members);
        members.reverse();
        members_optionals.reverse();
        member_ranges.reverse();
        Some(MemberExpressionInfo::Expression(ExpressionExpressionInfo {
          name,
          root_info: root_info.map_or_else(
            || ExportedVariableInfo::Name(Atom::from(root_name)),
            |i| ExportedVariableInfo::VariableInfo(i.id()),
          ),
          members,
          members_optionals,
          member_ranges,
        }))
      }
      _ => None,
    }
  }

  pub fn get_member_expression_info_from_expr(
    &mut self,
    expr: Expr,
    allowed_types: AllowedMemberTypes,
  ) -> Option<MemberExpressionInfo> {
    let expr_ref = ExprRef::from_expr(self.ast.ast, expr);
    match expr_ref {
      ExprRef::Member(_) | ExprRef::OptChain(_) => {
        self.get_member_expression_info(expr_ref, allowed_types)
      }
      _ => self._get_member_expression_info(
        expr_ref,
        RawAtomMembers::new(),
        OptionalMembers::new(),
        MemberRanges::new(),
        allowed_types,
      ),
    }
  }

  pub fn get_member_expression_info(
    &mut self,
    expr: ExprRef,
    allowed_types: AllowedMemberTypes,
  ) -> Option<MemberExpressionInfo> {
    let RawExtractedMemberExpressionChainData {
      object,
      members,
      members_optionals,
      member_ranges,
    } = self.extract_member_expression_chain_raw(expr);
    self._get_member_expression_info(
      object,
      members,
      members_optionals,
      member_ranges,
      allowed_types,
    )
  }

  pub fn extract_member_expression_chain(
    &self,
    expr: ExprRef,
  ) -> ExtractedMemberExpressionChainData {
    let RawExtractedMemberExpressionChainData {
      object,
      members,
      members_optionals,
      member_ranges,
    } = self.extract_member_expression_chain_raw(expr);
    ExtractedMemberExpressionChainData {
      object,
      members: materialize_member_atoms(self.ast.ast, members),
      members_optionals,
      member_ranges,
    }
  }

  fn extract_member_expression_chain_raw(
    &self,
    expr: ExprRef,
  ) -> RawExtractedMemberExpressionChainData {
    let ast = self.ast.ast;
    let mut object = expr;
    let mut members = RawAtomMembers::new();
    let mut members_optionals = OptionalMembers::new();
    let mut member_ranges = MemberRanges::new();
    let mut in_optional_chain = self.member_expr_in_optional_chain;
    loop {
      match object {
        ExprRef::Member(expr) => {
          let property = expr.property(ast);
          let property_data = ast.property_key_data(property);
          let object_expr = expr.object(ast);
          if expr.computed(ast) {
            if !member_property_key_data_can_be_atom(ast, property_data) {
              break;
            }
            members.push(property_data);
          } else if matches!(property_data, PropertyKeyData::IdentifierName(_)) {
            members.push(property_data);
          } else {
            break;
          }
          member_ranges.push(object_expr.span(ast));
          members_optionals.push(in_optional_chain || expr.optional(ast));
          object = ExprRef::from_expr(ast, object_expr);
          in_optional_chain = false;
        }
        ExprRef::OptChain(expr) => {
          let expression = expr.expression(ast);
          if let Some(member) = expression.as_member_expression(ast) {
            in_optional_chain = member.optional(ast);
            object = ExprRef::Member(member);
          } else {
            break;
          }
        }
        _ => break,
      }
    }
    RawExtractedMemberExpressionChainData {
      object,
      members,
      members_optionals,
      member_ranges,
    }
  }

  fn enter_ident<F>(&mut self, ident: BindingIdentifier, on_ident: F)
  where
    F: FnOnce(&mut Self, BindingIdentifier),
  {
    let ast = self.ast.ast;
    let name = ast.get_utf8(ident.name(ast));
    let drive = self.plugin_drive.clone();
    if !name
      .call_hooks_name(self, |parser, for_name| {
        drive.pattern(parser, ident, for_name)
      })
      .unwrap_or_default()
    {
      on_ident(self, ident);
    }
  }

  fn enter_array_pattern<F>(&mut self, array_pat: ArrayPattern, on_ident: F)
  where
    F: FnOnce(&mut Self, BindingIdentifier) + Copy,
  {
    let ast = self.ast.ast;
    for slot in array_pat.elements(ast).iter() {
      if let Some(element) = ast.get_node_in_sub_range(slot) {
        self.enter_pattern(PatRef::Borrowed(element), on_ident);
      }
    }
    if let Some(rest) = array_pat.rest(ast) {
      self.enter_rest_pattern(rest, on_ident);
    }
  }

  fn enter_assignment_pattern<F>(&mut self, assign: AssignmentPattern, on_ident: F)
  where
    F: FnOnce(&mut Self, BindingIdentifier) + Copy,
  {
    self.enter_pattern(PatRef::Borrowed(assign.left(self.ast.ast)), on_ident);
  }

  fn enter_object_pattern<F>(&mut self, object: ObjectPattern, on_ident: F)
  where
    F: FnOnce(&mut Self, BindingIdentifier) + Copy,
  {
    let ast = self.ast.ast;
    for slot in object.properties(ast).iter() {
      let property = ast.get_node_in_sub_range(slot);
      let value = property.value(ast);
      let old = self.in_short_hand;
      if property.shorthand(ast) && !value.is_assignment_pattern(ast) {
        self.in_short_hand = true;
      }
      self.enter_pattern(PatRef::Borrowed(value), on_ident);
      self.in_short_hand = old;
    }
    if let Some(rest) = object.rest(ast) {
      self.enter_rest_pattern(rest, on_ident);
    }
  }

  fn enter_rest_pattern<F>(&mut self, rest: BindingRestElement, on_ident: F)
  where
    F: FnOnce(&mut Self, BindingIdentifier) + Copy,
  {
    self.enter_pattern(PatRef::Borrowed(rest.argument(self.ast.ast)), on_ident)
  }

  fn enter_pattern<F>(&mut self, pattern: PatRef, on_ident: F)
  where
    F: FnOnce(&mut Self, BindingIdentifier) + Copy,
  {
    match self.ast.ast.binding_pattern_data(pattern.as_pat()) {
      BindingPatternData::BindingIdentifier(ident) => self.enter_ident(ident, on_ident),
      BindingPatternData::ArrayPattern(array) => self.enter_array_pattern(array, on_ident),
      BindingPatternData::AssignmentPattern(assign) => {
        self.enter_assignment_pattern(assign, on_ident)
      }
      BindingPatternData::ObjectPattern(object) => self.enter_object_pattern(object, on_ident),
      BindingPatternData::BindingRestElement(rest) => self.enter_rest_pattern(rest, on_ident),
      BindingPatternData::SimpleAssignmentTarget(_) => (),
    }
  }

  fn enter_patterns<I, F>(&mut self, patterns: I, on_ident: F)
  where
    F: FnOnce(&mut Self, BindingIdentifier) + Copy,
    I: Iterator<Item = PatRef>,
  {
    for pattern in patterns {
      self.enter_pattern(pattern, on_ident);
    }
  }

  fn enter_optional_chain<C, M, R>(
    &mut self,
    expr: swc_next_ecma_ast::ChainExpression,
    on_call: C,
    on_member: M,
  ) -> R
  where
    C: FnOnce(&mut Self, CallExpression) -> R,
    M: FnOnce(&mut Self, MemberExpression) -> R,
  {
    let member_expr_in_optional_chain = self.member_expr_in_optional_chain;
    let ast = self.ast.ast;
    let expression = expr.expression(ast);
    let ret = if let Some(call) = expression.as_call_expression(ast) {
      if call.callee(ast).is_member_expression(ast) {
        self.member_expr_in_optional_chain = call.optional(ast);
      }
      on_call(self, call)
    } else if let Some(member) = expression.as_member_expression(ast) {
      self.member_expr_in_optional_chain = member.optional(ast);
      on_member(self, member)
    } else {
      unreachable!("chain expression must contain a call or member expression")
    };
    self.member_expr_in_optional_chain = member_expr_in_optional_chain;
    ret
  }

  fn enter_declaration<F>(&mut self, decl: Decl, on_ident: F)
  where
    F: FnOnce(&mut Self, BindingIdentifier) + Copy,
  {
    let ast = self.ast.ast;
    match ast.decl_data(decl) {
      DeclData::Class(class) => {
        if let Some(id) = class.id(ast) {
          self.enter_ident(id, on_ident);
        }
      }
      DeclData::Function(function) => {
        if let Some(id) = function.id(ast) {
          self.enter_ident(id, on_ident);
        }
      }
      DeclData::VariableDeclaration(variable) => {
        for slot in variable.declarators(ast).iter() {
          let declarator = ast.get_node_in_sub_range(slot);
          self.enter_pattern(PatRef::Borrowed(declarator.id(ast)), on_ident);
        }
      }
      _ => (),
    }
  }

  fn enter_statement<S, H, F>(&mut self, span: Span, statement: S, call_hook: H, on_statement: F)
  where
    S: Copy,
    H: FnOnce(&mut Self, S) -> bool,
    F: FnOnce(&mut Self, S),
  {
    self.statement_path.push(span.into());
    if call_hook(self, statement) {
      self.prev_statement = self.statement_path.pop();
      return;
    }
    on_statement(self, statement);
    self.prev_statement = self.statement_path.pop();
  }

  pub fn enter_destructuring_assignment(
    &mut self,
    pattern: ObjectPattern,
    expr: Expr,
  ) -> Option<Expr> {
    let ast = self.ast.ast;
    let drive = self.plugin_drive.clone();
    let expr = if let Some(await_expr) = expr.as_await_expression(ast) {
      await_expr.argument(ast)
    } else {
      expr
    };
    let destructuring = if let Some(assign) = expr.as_assignment_expression(ast)
      && let Some(obj_pat) = assign.left(ast).as_object_assignment_target(ast)
    {
      self.enter_destructuring_assignment(obj_pat, assign.right(ast))
    } else {
      let can_collect = drive
        .can_collect_destructuring_assignment_properties(self, expr)
        .unwrap_or_default();
      can_collect.then_some(expr)
    };
    let destructuring_span = destructuring.map(|destructuring| destructuring.span(ast));
    if let Some(destructuring_span) = destructuring_span
      && let Some(keys) =
        self.collect_destructuring_assignment_properties_from_object_pattern(pattern)
    {
      self
        .destructuring_assignment_properties
        .add(destructuring_span, keys);
    }
    destructuring
  }

  pub fn walk_program(&mut self, program: Program) {
    let drive = self.plugin_drive.clone();
    if drive.program(self, program).is_none() {
      let ast = self.ast.ast;
      let body = program.body(ast);
      // Match the legacy `Program::Module` traversal without treating an
      // `import.meta`-only unambiguous parse as ESM. Do not set `self.is_esm`
      // early: legacy parsing only flipped that state during pre-walk.
      let is_esm_program = matches!(self.module_type, ModuleType::JsEsm)
        || body.iter().any(|slot| {
          let statement = ast.get_node_in_sub_range(slot);
          matches!(
            ast.stmt_data(statement),
            StmtData::ImportDeclaration(_)
              | StmtData::ExportNamedDeclaration(_)
              | StmtData::ExportDefaultDeclaration(_)
              | StmtData::ExportAllDeclaration(_)
              | StmtData::TsExportAssignment(_)
              | StmtData::TsNamespaceExportDeclaration(_)
          )
        });
      if is_esm_program {
        self.set_strict(true);
        self.prev_statement = None;
        self.module_pre_walk_module_items(body);
      } else {
        self.detect_mode(program);
      }
      self.prev_statement = None;
      self.pre_walk_module_items(body);
      self.prev_statement = None;
      self.block_pre_walk_module_items(body);
      self.prev_statement = None;
      self.walk_module_items(body);
    }
    drive.finish(self);
  }

  fn set_strict(&mut self, value: bool) {
    let current_scope = self.definitions_db.expect_get_mut_scope(self.definitions);
    current_scope.is_strict = value;
  }

  pub fn detect_mode(&mut self, program: Program) {
    let ast = self.ast.ast;
    for slot in program.directives(ast).iter() {
      let directive = ast.get_node_in_sub_range(slot);
      if ast.get_utf8(directive.value(ast)) == "use strict" {
        self.set_strict(true);
        return;
      }
    }
  }

  pub fn is_strict(&mut self) -> bool {
    let scope = self.definitions_db.expect_get_scope(self.definitions);
    scope.is_strict
  }

  pub fn is_variable_defined(&mut self, name: &str) -> bool {
    let Some(info) = self.get_variable_info(name) else {
      return false;
    };
    !info.is_free()
  }
}

impl<'parser> JavascriptParser<'parser> {
  pub fn evaluate_expression(&mut self, expr: Expr) -> BasicEvaluatedExpression<'parser> {
    let span = expr.span(self.ast.ast);
    match self.evaluating(expr) {
      Some(evaluated) => evaluated.with_expression(Some(expr)),
      None => BasicEvaluatedExpression::with_range(span.real_lo(), span.real_hi())
        .with_expression(Some(expr)),
    }
  }

  pub fn evaluate<T: Display>(
    &mut self,
    source: String,
    error_title: T,
  ) -> Option<BasicEvaluatedExpression<'parser>> {
    eval::eval_source(self, source, error_title.to_string())
  }

  // same as `JavascriptParser._initializeEvaluating` in webpack
  // FIXME: should mv it to plugin(for example `parse.hooks.evaluate for`)
  fn evaluating(&mut self, expr: Expr) -> Option<BasicEvaluatedExpression<'parser>> {
    let ast = self.ast.ast;
    match ast.expr_data(expr) {
      ExprData::TemplateLiteral(tpl) => eval::eval_tpl_expression(self, tpl),
      ExprData::TaggedTemplateExpression(tagged_tpl) => {
        eval::eval_tagged_tpl_expression(self, tagged_tpl)
      }
      ExprData::StringLiteral(_)
      | ExprData::NumericLiteral(_)
      | ExprData::BigIntLiteral(_)
      | ExprData::BooleanLiteral(_)
      | ExprData::NullLiteral(_)
      | ExprData::RegExpLiteral(_) => eval::eval_lit_expr(ast, expr),
      ExprData::ConditionalExpression(cond) => eval::eval_cond_expression(self, cond),
      ExprData::UnaryExpression(unary) => eval::eval_unary_expression(self, unary),
      ExprData::BinaryExpression(binary) => eval::eval_binary_expression(self, binary),
      ExprData::LogicalExpression(logical) => eval::eval_logical_expression(self, logical),
      ExprData::ArrayExpression(array) => eval::eval_array_expression(self, array),
      ExprData::NewExpression(new) => eval::eval_new_expression(self, new),
      ExprData::CallExpression(call) => eval::eval_call_expression(self, call),
      ExprData::ChainExpression(chain) => {
        let inner = chain.expression(ast);
        match ast.expr_data(inner) {
          ExprData::CallExpression(call) => eval::eval_call_expression(self, call),
          ExprData::MemberExpression(member) => eval::eval_member_expression(self, member, expr),
          _ => None,
        }
      }
      ExprData::MemberExpression(member) => eval::eval_member_expression(self, member, expr),
      ExprData::IdentifierReference(ident) => {
        let span = ident.span(ast);
        let name = ast.get_utf8(ident.name(ast));
        if name == "undefined" {
          let mut eval = BasicEvaluatedExpression::with_range(span.real_lo(), span.real_hi());
          eval.set_undefined();
          return Some(eval);
        }
        let drive = self.plugin_drive.clone();
        name
          .call_hooks_name(self, |parser, name| {
            drive.evaluate_identifier(parser, name, None, span.real_lo(), span.real_hi())
          })
          .or_else(|| {
            let info = self.get_variable_info(name);
            if let Some(info) = info {
              if let Some(name) = &info.name
                && (info.is_free() || info.is_tagged())
              {
                let mut eval = BasicEvaluatedExpression::with_range(span.real_lo(), span.real_hi());
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
              let name = Atom::from(name);
              let mut eval = BasicEvaluatedExpression::with_range(span.real_lo(), span.real_hi());
              eval.set_identifier(
                name.clone(),
                ExportedVariableInfo::Name(name.clone()),
                None,
                None,
                None,
              );
              Some(eval)
            }
          })
      }
      ExprData::ThisExpression(this) => {
        let span = this.span(ast);
        let drive = self.plugin_drive.clone();
        let default_eval = || {
          let mut eval = BasicEvaluatedExpression::with_range(span.real_lo(), span.real_hi());
          eval.set_identifier(
            "this".into(),
            ExportedVariableInfo::Name("this".into()),
            None,
            None,
            None,
          );
          Some(eval)
        };
        let Some(info) = self.get_variable_info("this") else {
          // use `ident.sym` as fallback for global variable(or maybe just a undefined variable)
          return drive
            .evaluate_identifier(self, "this", None, span.real_lo(), span.real_hi())
            .or_else(default_eval);
        };
        if let Some(name) = &info.name
          && (info.is_free() || info.is_tagged())
        {
          let name = name.clone();
          return drive
            .evaluate_identifier(self, &name, None, span.real_lo(), span.real_hi())
            .or_else(default_eval);
        }
        None
      }
      _ => None,
    }
  }

  pub fn to_dependency_location(&mut self, range: DependencyRange) -> Option<DependencyLocation> {
    self
      .location_advancer
      .compute_dependency_location(self.source, range)
  }
}
