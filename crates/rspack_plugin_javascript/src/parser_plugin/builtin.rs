use rspack_core::{
  BuildInfo, CompilerOptions, ImportMeta, JavascriptParserCommonjsExportsOption,
  JavascriptParserOptions, ModuleType,
};
use swc_core::{
  atoms::Atom,
  common::{Mark, Span},
  ecma::ast::{
    AssignExpr, AwaitExpr, BinExpr, CallExpr, Callee, ClassMember, CondExpr, Expr, ForOfStmt,
    Ident, IfStmt, ImportDecl, MemberExpr, ModuleDecl, NewExpr, Program, ThisExpr, UnaryExpr,
    UnaryOp, VarDeclarator,
  },
};

use super::{
  AMDDefineDependencyParserPlugin, AMDParserPlugin, AMDRequireDependenciesBlockParserPlugin,
  APIPlugin, CheckVarDeclaratorIdent, CommonJsExportsParserPlugin, CommonJsImportsParserPlugin,
  CommonJsPlugin, CompatibilityPlugin, ConstPlugin, ESMDetectionParserPlugin,
  ESMExportDependencyParserPlugin, ESMImportDependencyParserPlugin, ESMTopLevelThisParserPlugin,
  ExportsInfoApiPlugin, ImportMetaContextDependencyParserPlugin, ImportMetaDisabledPlugin,
  ImportMetaPlugin, ImportParserPlugin, InitializeEvaluating, InlineConstPlugin,
  InnerGraphParserPlugin, IsIncludedPlugin, JavascriptMetaInfoPlugin, JavascriptParserPlugin,
  JavascriptParserPluginHook, JavascriptParserPluginHooks, NodeStuffPlugin, OverrideStrictPlugin,
  RequireContextDependencyParserPlugin, RequireEnsureDependenciesBlockParserPlugin,
  SideEffectsParserPlugin, UseStrictPlugin, WorkerPlugin, r#const::is_logic_op,
};
use crate::{
  utils::eval::BasicEvaluatedExpression,
  visitors::{
    ClassDeclOrExpr, DestructuringAssignmentProperty, ExportDefaultDeclaration,
    ExportDefaultExpression, ExportImport, ExportLocal, ExportedVariableInfo, JavascriptParser,
    Statement, VariableDeclaration,
  },
};

pub enum ImportMetaParserPlugin {
  Enabled(ImportMetaPlugin),
  Disabled(ImportMetaDisabledPlugin),
}

macro_rules! dispatch_import_meta_plugin {
  ($self:expr, $method:ident($($arg:expr),* $(,)?)) => {{
    match $self {
      ImportMetaParserPlugin::Enabled(plugin) => JavascriptParserPlugin::$method(plugin, $($arg),*),
      ImportMetaParserPlugin::Disabled(plugin) => {
        JavascriptParserPlugin::$method(plugin, $($arg),*)
      }
    }
  }};
}

impl JavascriptParserPlugin for ImportMetaParserPlugin {
  fn implemented_hooks(&self) -> JavascriptParserPluginHooks {
    match self {
      ImportMetaParserPlugin::Enabled(plugin) => plugin.implemented_hooks(),
      ImportMetaParserPlugin::Disabled(_) => {
        JavascriptParserPluginHooks::empty().with(JavascriptParserPluginHook::MetaProperty)
      }
    }
  }

  fn evaluate_typeof<'a>(
    &self,
    parser: &mut JavascriptParser,
    expr: &'a UnaryExpr,
    for_name: &str,
  ) -> Option<BasicEvaluatedExpression<'a>> {
    dispatch_import_meta_plugin!(self, evaluate_typeof(parser, expr, for_name))
  }

  fn evaluate_identifier(
    &self,
    parser: &mut JavascriptParser,
    for_name: &str,
    start: u32,
    end: u32,
  ) -> Option<BasicEvaluatedExpression<'static>> {
    dispatch_import_meta_plugin!(self, evaluate_identifier(parser, for_name, start, end))
  }

  fn evaluate<'a>(
    &self,
    parser: &mut JavascriptParser,
    expr: &'a Expr,
  ) -> Option<BasicEvaluatedExpression<'a>> {
    dispatch_import_meta_plugin!(self, evaluate(parser, expr))
  }

  fn r#typeof(
    &self,
    parser: &mut JavascriptParser,
    expr: &UnaryExpr,
    for_name: &str,
  ) -> Option<bool> {
    dispatch_import_meta_plugin!(self, r#typeof(parser, expr, for_name))
  }

  fn can_collect_destructuring_assignment_properties(
    &self,
    parser: &mut JavascriptParser,
    expr: &Expr,
  ) -> Option<bool> {
    dispatch_import_meta_plugin!(
      self,
      can_collect_destructuring_assignment_properties(parser, expr)
    )
  }

  fn meta_property(
    &self,
    parser: &mut JavascriptParser,
    root_name: &Atom,
    span: Span,
  ) -> Option<bool> {
    dispatch_import_meta_plugin!(self, meta_property(parser, root_name, span))
  }

  fn member(
    &self,
    parser: &mut JavascriptParser,
    expr: &MemberExpr,
    for_name: &str,
  ) -> Option<bool> {
    dispatch_import_meta_plugin!(self, member(parser, expr, for_name))
  }

  fn call(&self, parser: &mut JavascriptParser, expr: &CallExpr, for_name: &str) -> Option<bool> {
    dispatch_import_meta_plugin!(self, call(parser, expr, for_name))
  }

  fn unhandled_expression_member_chain(
    &self,
    parser: &mut JavascriptParser,
    root_info: &ExportedVariableInfo,
    expr: &MemberExpr,
  ) -> Option<bool> {
    dispatch_import_meta_plugin!(
      self,
      unhandled_expression_member_chain(parser, root_info, expr)
    )
  }
}

macro_rules! dispatch_void_plugin {
  ($self:expr, opt $plugin:ident, $method:ident($($arg:expr),* $(,)?)) => {{
    if let Some(plugin) = $self.$plugin.as_ref() {
      JavascriptParserPlugin::$method(plugin, $($arg),*);
    }
  }};
}

macro_rules! dispatch_bail_plugin {
  ($self:expr, opt $plugin:ident, $method:ident($($arg:expr),* $(,)?)) => {{
    if let Some(plugin) = $self.$plugin.as_ref() {
      let res = JavascriptParserPlugin::$method(plugin, $($arg),*);
      if res.is_some() {
        return res;
      }
    }
  }};
  ($self:expr, req $plugin:ident, $method:ident($($arg:expr),* $(,)?)) => {{
    let res = JavascriptParserPlugin::$method(&$self.$plugin, $($arg),*);
    if res.is_some() {
      return res;
    }
  }};
}

macro_rules! dispatch_eval_bail_plugin {
  ($self:expr, req $plugin:ident, $method:ident($($arg:expr),* $(,)?), $param:expr) => {{
    let res = JavascriptParserPlugin::$method(&$self.$plugin, $($arg),*, $param.clone());
    if res.is_some() {
      return res;
    }
  }};
}

macro_rules! dispatch_void {
  ($self:expr, [$($kind:ident $plugin:ident),* $(,)?], $method:ident $args:tt) => {{
    $(dispatch_void_plugin!($self, $kind $plugin, $method $args);)*
  }};
}

macro_rules! dispatch_bail {
  ($self:expr, [$($kind:ident $plugin:ident),* $(,)?], $method:ident $args:tt) => {{
    $(dispatch_bail_plugin!($self, $kind $plugin, $method $args);)*
    None
  }};
}

macro_rules! dispatch_eval_bail {
  ($self:expr, [$($kind:ident $plugin:ident),* $(,)?], $method:ident $args:tt, $param:expr) => {{
    $(dispatch_eval_bail_plugin!($self, $kind $plugin, $method $args, $param);)*
    None
  }};
}

pub struct BuiltinJavascriptParserPlugin {
  initialize_evaluating: InitializeEvaluating,
  javascript_meta_info: JavascriptMetaInfoPlugin,
  check_var_declarator_ident: CheckVarDeclaratorIdent,
  const_plugin: ConstPlugin,
  use_strict: UseStrictPlugin,
  require_context_dependency: RequireContextDependencyParserPlugin,
  require_ensure_dependencies_block: RequireEnsureDependenciesBlockParserPlugin,
  compatibility: CompatibilityPlugin,
  esm_top_level_this: Option<ESMTopLevelThisParserPlugin>,
  esm_detection: Option<ESMDetectionParserPlugin>,
  import_meta_context_dependency: Option<ImportMetaContextDependencyParserPlugin>,
  import_meta: Option<ImportMetaParserPlugin>,
  esm_import_dependency: Option<ESMImportDependencyParserPlugin>,
  esm_export_dependency: Option<ESMExportDependencyParserPlugin>,
  amd_require_dependencies_block: Option<AMDRequireDependenciesBlockParserPlugin>,
  amd_define_dependency: Option<AMDDefineDependencyParserPlugin>,
  amd_parser: Option<AMDParserPlugin>,
  common_js_imports: Option<CommonJsImportsParserPlugin>,
  common_js: Option<CommonJsPlugin>,
  common_js_exports: Option<CommonJsExportsParserPlugin>,
  node: Option<NodeStuffPlugin>,
  is_included: Option<IsIncludedPlugin>,
  exports_info_api: Option<ExportsInfoApiPlugin>,
  api: Option<APIPlugin>,
  import_parser: Option<ImportParserPlugin>,
  worker: Option<WorkerPlugin>,
  override_strict: Option<OverrideStrictPlugin>,
  inline_const: Option<InlineConstPlugin>,
  inner_graph: Option<InnerGraphParserPlugin>,
  side_effects: Option<SideEffectsParserPlugin>,
}

impl BuiltinJavascriptParserPlugin {
  pub fn new(
    compiler_options: &CompilerOptions,
    javascript_options: &JavascriptParserOptions,
    module_type: &ModuleType,
    unresolved_mark: Mark,
    build_info: &mut BuildInfo,
  ) -> Self {
    let is_esm = module_type.is_js_auto() || module_type.is_js_esm();
    let is_common_js = module_type.is_js_auto() || module_type.is_js_dynamic();
    let is_javascript_module = is_esm || module_type.is_js_dynamic();

    let import_meta = is_esm.then(|| {
      if matches!(
        javascript_options.import_meta,
        Some(ImportMeta::Enabled | ImportMeta::PreserveUnknown)
      ) {
        ImportMetaParserPlugin::Enabled(ImportMetaPlugin(
          javascript_options.import_meta.expect("should have value"),
        ))
      } else {
        ImportMetaParserPlugin::Disabled(ImportMetaDisabledPlugin)
      }
    });

    let amd_enabled =
      compiler_options.amd.is_some() && (module_type.is_js_auto() || module_type.is_js_dynamic());

    let common_js_exports = if is_common_js {
      let commonjs_exports = javascript_options
        .commonjs
        .as_ref()
        .map_or(JavascriptParserCommonjsExportsOption::Enable, |commonjs| {
          commonjs.exports
        });
      (commonjs_exports != JavascriptParserCommonjsExportsOption::Disable).then(|| {
        CommonJsExportsParserPlugin::new(
          commonjs_exports == JavascriptParserCommonjsExportsOption::SkipInEsm,
        )
      })
    } else {
      None
    };

    let handle_cjs = is_common_js && compiler_options.node.is_some();
    let handle_esm = is_esm;
    let node = (handle_cjs || handle_esm).then(|| NodeStuffPlugin::new(handle_cjs, handle_esm));

    let inline_const = compiler_options.optimization.inline_exports.then(|| {
      build_info.inline_exports = true;
      InlineConstPlugin
    });

    Self {
      initialize_evaluating: InitializeEvaluating,
      javascript_meta_info: JavascriptMetaInfoPlugin,
      check_var_declarator_ident: CheckVarDeclaratorIdent,
      const_plugin: ConstPlugin,
      use_strict: UseStrictPlugin,
      require_context_dependency: RequireContextDependencyParserPlugin,
      require_ensure_dependencies_block: RequireEnsureDependenciesBlockParserPlugin,
      compatibility: CompatibilityPlugin,
      esm_top_level_this: is_esm.then_some(ESMTopLevelThisParserPlugin),
      esm_detection: is_esm.then_some(ESMDetectionParserPlugin),
      import_meta_context_dependency: is_esm.then_some(ImportMetaContextDependencyParserPlugin),
      import_meta,
      esm_import_dependency: is_esm.then_some(ESMImportDependencyParserPlugin),
      esm_export_dependency: is_esm.then_some(ESMExportDependencyParserPlugin),
      amd_require_dependencies_block: amd_enabled
        .then_some(AMDRequireDependenciesBlockParserPlugin),
      amd_define_dependency: amd_enabled.then_some(AMDDefineDependencyParserPlugin),
      amd_parser: amd_enabled.then_some(AMDParserPlugin),
      common_js_imports: is_common_js.then_some(CommonJsImportsParserPlugin),
      common_js: is_common_js.then_some(CommonJsPlugin),
      common_js_exports,
      node,
      is_included: is_javascript_module.then_some(IsIncludedPlugin),
      exports_info_api: is_javascript_module.then_some(ExportsInfoApiPlugin),
      api: is_javascript_module.then(|| APIPlugin::new(compiler_options.output.module)),
      import_parser: is_javascript_module.then_some(ImportParserPlugin),
      worker: is_javascript_module.then(|| {
        WorkerPlugin::new(
          javascript_options
            .worker
            .as_ref()
            .expect("should have worker"),
        )
      }),
      override_strict: is_javascript_module.then_some(OverrideStrictPlugin),
      inline_const,
      inner_graph: compiler_options.optimization.inner_graph.then(|| {
        InnerGraphParserPlugin::new(unresolved_mark, compiler_options.experiments.pure_functions)
      }),
      side_effects: compiler_options
        .optimization
        .side_effects
        .is_true()
        .then(|| {
          SideEffectsParserPlugin::new(unresolved_mark, compiler_options.experiments.pure_functions)
        }),
    }
  }
}

impl JavascriptParserPlugin for BuiltinJavascriptParserPlugin {
  fn implemented_hooks(&self) -> JavascriptParserPluginHooks {
    let mut hooks = self
      .initialize_evaluating
      .implemented_hooks()
      .union(self.javascript_meta_info.implemented_hooks())
      .union(self.check_var_declarator_ident.implemented_hooks())
      .union(self.const_plugin.implemented_hooks())
      .union(self.use_strict.implemented_hooks())
      .union(self.require_context_dependency.implemented_hooks())
      .union(self.require_ensure_dependencies_block.implemented_hooks())
      .union(self.compatibility.implemented_hooks());

    if let Some(plugin) = self.esm_top_level_this.as_ref() {
      hooks = hooks.union(plugin.implemented_hooks());
    }
    if let Some(plugin) = self.esm_detection.as_ref() {
      hooks = hooks.union(plugin.implemented_hooks());
    }
    if let Some(plugin) = self.import_meta_context_dependency.as_ref() {
      hooks = hooks.union(plugin.implemented_hooks());
    }
    if let Some(plugin) = self.import_meta.as_ref() {
      hooks = hooks.union(plugin.implemented_hooks());
    }
    if let Some(plugin) = self.esm_import_dependency.as_ref() {
      hooks = hooks.union(plugin.implemented_hooks());
    }
    if let Some(plugin) = self.esm_export_dependency.as_ref() {
      hooks = hooks.union(plugin.implemented_hooks());
    }
    if let Some(plugin) = self.amd_require_dependencies_block.as_ref() {
      hooks = hooks.union(plugin.implemented_hooks());
    }
    if let Some(plugin) = self.amd_define_dependency.as_ref() {
      hooks = hooks.union(plugin.implemented_hooks());
    }
    if let Some(plugin) = self.amd_parser.as_ref() {
      hooks = hooks.union(plugin.implemented_hooks());
    }
    if let Some(plugin) = self.common_js_imports.as_ref() {
      hooks = hooks.union(plugin.implemented_hooks());
    }
    if let Some(plugin) = self.common_js.as_ref() {
      hooks = hooks.union(plugin.implemented_hooks());
    }
    if let Some(plugin) = self.common_js_exports.as_ref() {
      hooks = hooks.union(plugin.implemented_hooks());
    }
    if let Some(plugin) = self.node.as_ref() {
      hooks = hooks.union(plugin.implemented_hooks());
    }
    if let Some(plugin) = self.is_included.as_ref() {
      hooks = hooks.union(plugin.implemented_hooks());
    }
    if let Some(plugin) = self.exports_info_api.as_ref() {
      hooks = hooks.union(plugin.implemented_hooks());
    }
    if let Some(plugin) = self.api.as_ref() {
      hooks = hooks.union(plugin.implemented_hooks());
    }
    if let Some(plugin) = self.import_parser.as_ref() {
      hooks = hooks.union(plugin.implemented_hooks());
    }
    if let Some(plugin) = self.worker.as_ref() {
      hooks = hooks.union(plugin.implemented_hooks());
    }
    if let Some(plugin) = self.override_strict.as_ref() {
      hooks = hooks.union(plugin.implemented_hooks());
    }
    if let Some(plugin) = self.inline_const.as_ref() {
      hooks = hooks.union(plugin.implemented_hooks());
    }
    if let Some(plugin) = self.inner_graph.as_ref() {
      hooks = hooks.union(plugin.implemented_hooks());
    }
    if let Some(plugin) = self.side_effects.as_ref() {
      hooks = hooks.union(plugin.implemented_hooks());
    }

    hooks
  }

  fn top_level_await_expr(&self, parser: &mut JavascriptParser, expr: &AwaitExpr) {
    dispatch_void!(self, [opt esm_detection], top_level_await_expr(parser, expr));
  }

  fn top_level_for_of_await_stmt(&self, parser: &mut JavascriptParser, stmt: &ForOfStmt) {
    dispatch_void!(
      self,
      [opt esm_detection],
      top_level_for_of_await_stmt(parser, stmt)
    );
  }

  fn program(&self, parser: &mut JavascriptParser, ast: &Program) -> Option<bool> {
    dispatch_bail!(
      self,
      [
        req use_strict,
        req compatibility,
        opt esm_detection,
        opt override_strict,
        opt inline_const,
        opt inner_graph,
        opt side_effects,
      ],
      program(parser, ast)
    )
  }

  fn finish(&self, parser: &mut JavascriptParser) -> Option<bool> {
    dispatch_bail!(
      self,
      [
        req javascript_meta_info,
        opt amd_define_dependency,
        opt common_js_imports,
        opt import_parser,
        opt inner_graph,
        opt side_effects,
      ],
      finish(parser)
    )
  }

  fn block_pre_module_declaration(
    &self,
    parser: &mut JavascriptParser,
    decl: &ModuleDecl,
  ) -> Option<bool> {
    dispatch_bail!(
      self,
      [opt inner_graph],
      block_pre_module_declaration(parser, decl)
    )
  }

  fn module_declaration(&self, parser: &mut JavascriptParser, decl: &ModuleDecl) -> Option<bool> {
    dispatch_bail!(
      self,
      [opt esm_export_dependency, opt inner_graph, opt side_effects],
      module_declaration(parser, decl)
    )
  }

  fn call(&self, parser: &mut JavascriptParser, expr: &CallExpr, name: &str) -> Option<bool> {
    dispatch_bail!(
      self,
      [
        req javascript_meta_info,
        req require_context_dependency,
        req require_ensure_dependencies_block,
        req compatibility,
        opt esm_detection,
        opt import_meta_context_dependency,
        opt import_meta,
        opt amd_require_dependencies_block,
        opt amd_define_dependency,
        opt amd_parser,
        opt common_js_imports,
        opt common_js_exports,
        opt is_included,
        opt api,
        opt worker,
      ],
      call(parser, expr, name)
    )
  }

  fn member(
    &self,
    parser: &mut JavascriptParser,
    expr: &MemberExpr,
    for_name: &str,
  ) -> Option<bool> {
    dispatch_bail!(
      self,
      [
        opt import_meta,
        opt amd_parser,
        opt common_js,
        opt common_js_exports,
        opt node,
        opt api,
        opt inner_graph,
      ],
      member(parser, expr, for_name)
    )
  }

  fn member_chain(
    &self,
    parser: &mut JavascriptParser,
    expr: &MemberExpr,
    for_name: &str,
    members: &[Atom],
    members_optionals: &[bool],
    member_ranges: &[Span],
  ) -> Option<bool> {
    dispatch_bail!(
      self,
      [
        opt esm_import_dependency,
        opt common_js_imports,
        opt common_js_exports,
        opt exports_info_api,
        opt import_parser,
      ],
      member_chain(parser, expr, for_name, members, members_optionals, member_ranges)
    )
  }

  fn call_member_chain(
    &self,
    parser: &mut JavascriptParser,
    expr: &CallExpr,
    for_name: &str,
    members: &[Atom],
    members_optionals: &[bool],
    member_ranges: &[Span],
  ) -> Option<bool> {
    assert!(matches!(expr.callee, Callee::Expr(_)));
    dispatch_bail!(
      self,
      [
        opt esm_import_dependency,
        opt common_js_imports,
        opt common_js_exports,
        opt import_parser,
        opt worker,
      ],
      call_member_chain(parser, expr, for_name, members, members_optionals, member_ranges)
    )
  }

  fn member_chain_of_call_member_chain(
    &self,
    parser: &mut JavascriptParser,
    member_expr: &MemberExpr,
    callee_members: &[Atom],
    call_expr: &CallExpr,
    members: &[Atom],
    member_ranges: &[Span],
    for_name: &str,
  ) -> Option<bool> {
    dispatch_bail!(
      self,
      [opt common_js_imports],
      member_chain_of_call_member_chain(
        parser,
        member_expr,
        callee_members,
        call_expr,
        members,
        member_ranges,
        for_name
      )
    )
  }

  fn call_member_chain_of_call_member_chain(
    &self,
    parser: &mut JavascriptParser,
    call_expr: &CallExpr,
    callee_members: &[Atom],
    inner_call_expr: &CallExpr,
    members: &[Atom],
    member_ranges: &[Span],
    for_name: &str,
  ) -> Option<bool> {
    dispatch_bail!(
      self,
      [opt common_js_imports],
      call_member_chain_of_call_member_chain(
        parser,
        call_expr,
        callee_members,
        inner_call_expr,
        members,
        member_ranges,
        for_name
      )
    )
  }

  fn assign(
    &self,
    parser: &mut JavascriptParser,
    expr: &AssignExpr,
    for_name: &str,
  ) -> Option<bool> {
    dispatch_bail!(
      self,
      [opt common_js_imports, opt inner_graph],
      assign(parser, expr, for_name)
    )
  }

  fn assign_member_chain(
    &self,
    parser: &mut JavascriptParser,
    expr: &AssignExpr,
    members: &[Atom],
    for_name: &str,
  ) -> Option<bool> {
    dispatch_bail!(
      self,
      [opt common_js_exports],
      assign_member_chain(parser, expr, members, for_name)
    )
  }

  fn r#typeof(
    &self,
    parser: &mut JavascriptParser,
    expr: &UnaryExpr,
    for_name: &str,
  ) -> Option<bool> {
    assert!(expr.op == UnaryOp::TypeOf);
    dispatch_bail!(
      self,
      [
        req require_ensure_dependencies_block,
        opt esm_detection,
        opt import_meta,
        opt amd_parser,
        opt common_js_imports,
        opt common_js,
        opt common_js_exports,
        opt node,
        opt is_included,
      ],
      r#typeof(parser, expr, for_name)
    )
  }

  fn expression_logical_operator(
    &self,
    parser: &mut JavascriptParser,
    expr: &BinExpr,
  ) -> Option<bool> {
    assert!(is_logic_op(expr.op));
    dispatch_bail!(
      self,
      [req const_plugin],
      expression_logical_operator(parser, expr)
    )
  }

  fn binary_expression(&self, parser: &mut JavascriptParser, expr: &BinExpr) -> Option<bool> {
    assert!(!is_logic_op(expr.op));
    dispatch_bail!(
      self,
      [opt esm_import_dependency],
      binary_expression(parser, expr)
    )
  }

  fn statement(&self, parser: &mut JavascriptParser, stmt: Statement) -> Option<bool> {
    dispatch_bail!(self, [opt inner_graph, opt side_effects], statement(parser, stmt))
  }

  fn unused_statement(&self, parser: &mut JavascriptParser, stmt: Statement) -> Option<bool> {
    dispatch_bail!(self, [req const_plugin], unused_statement(parser, stmt))
  }

  fn statement_if(&self, parser: &mut JavascriptParser, expr: &IfStmt) -> Option<bool> {
    dispatch_bail!(self, [req const_plugin], statement_if(parser, expr))
  }

  fn declarator(
    &self,
    parser: &mut JavascriptParser,
    expr: &VarDeclarator,
    stmt: VariableDeclaration<'_>,
  ) -> Option<bool> {
    dispatch_bail!(
      self,
      [req check_var_declarator_ident, opt inner_graph],
      declarator(parser, expr, stmt)
    )
  }

  fn new_expression(
    &self,
    parser: &mut JavascriptParser,
    expr: &NewExpr,
    for_name: &str,
  ) -> Option<bool> {
    dispatch_bail!(
      self,
      [opt common_js_imports, opt worker],
      new_expression(parser, expr, for_name)
    )
  }

  fn identifier(
    &self,
    parser: &mut JavascriptParser,
    expr: &Ident,
    for_name: &str,
  ) -> Option<bool> {
    dispatch_bail!(
      self,
      [
        req const_plugin,
        req compatibility,
        opt esm_detection,
        opt esm_import_dependency,
        opt amd_parser,
        opt common_js_imports,
        opt common_js_exports,
        opt node,
        opt exports_info_api,
        opt api,
        opt import_parser,
        opt inner_graph,
      ],
      identifier(parser, expr, for_name)
    )
  }

  fn class_extends_expression(
    &self,
    parser: &mut JavascriptParser,
    super_class: &Expr,
    class_decl_or_expr: ClassDeclOrExpr,
  ) -> Option<bool> {
    dispatch_bail!(
      self,
      [opt inner_graph],
      class_extends_expression(parser, super_class, class_decl_or_expr)
    )
  }

  fn class_body_element(
    &self,
    parser: &mut JavascriptParser,
    member: &ClassMember,
    class_decl_or_expr: ClassDeclOrExpr,
  ) -> Option<bool> {
    dispatch_bail!(
      self,
      [opt inner_graph],
      class_body_element(parser, member, class_decl_or_expr)
    )
  }

  fn class_body_value(
    &self,
    parser: &mut JavascriptParser,
    element: &ClassMember,
    expr_span: Span,
    class_decl_or_expr: ClassDeclOrExpr,
  ) -> Option<bool> {
    dispatch_bail!(
      self,
      [opt inner_graph],
      class_body_value(parser, element, expr_span, class_decl_or_expr)
    )
  }

  fn this(&self, parser: &mut JavascriptParser, expr: &ThisExpr, for_name: &str) -> Option<bool> {
    dispatch_bail!(
      self,
      [opt esm_top_level_this, opt common_js_exports, opt inner_graph],
      this(parser, expr, for_name)
    )
  }

  fn evaluate<'a>(
    &self,
    parser: &mut JavascriptParser,
    expr: &'a Expr,
  ) -> Option<BasicEvaluatedExpression<'a>> {
    dispatch_bail!(self, [opt import_meta], evaluate(parser, expr))
  }

  fn evaluate_typeof<'a>(
    &self,
    parser: &mut JavascriptParser,
    expr: &'a UnaryExpr,
    for_name: &str,
  ) -> Option<BasicEvaluatedExpression<'a>> {
    dispatch_bail!(
      self,
      [
        req require_ensure_dependencies_block,
        opt esm_detection,
        opt import_meta,
        opt amd_parser,
        opt common_js_imports,
        opt common_js_exports,
        opt node,
        opt api,
      ],
      evaluate_typeof(parser, expr, for_name)
    )
  }

  fn evaluate_call_expression<'a>(
    &self,
    parser: &mut JavascriptParser,
    name: &str,
    expr: &'a CallExpr,
  ) -> Option<BasicEvaluatedExpression<'a>> {
    dispatch_bail!(
      self,
      [req initialize_evaluating],
      evaluate_call_expression(parser, name, expr)
    )
  }

  fn evaluate_call_expression_member<'a>(
    &self,
    parser: &mut JavascriptParser,
    property: &str,
    expr: &'a CallExpr,
    param: BasicEvaluatedExpression<'a>,
  ) -> Option<BasicEvaluatedExpression<'a>> {
    dispatch_eval_bail!(
      self,
      [req initialize_evaluating],
      evaluate_call_expression_member(parser, property, expr),
      param
    )
  }

  fn evaluate_identifier(
    &self,
    parser: &mut JavascriptParser,
    for_name: &str,
    start: u32,
    end: u32,
  ) -> Option<BasicEvaluatedExpression<'static>> {
    dispatch_bail!(
      self,
      [
        req const_plugin,
        opt import_meta_context_dependency,
        opt import_meta,
        opt amd_parser,
        opt common_js_imports,
        opt common_js,
        opt node,
        opt api,
        opt inline_const,
      ],
      evaluate_identifier(parser, for_name, start, end)
    )
  }

  fn can_collect_destructuring_assignment_properties(
    &self,
    parser: &mut JavascriptParser,
    expr: &Expr,
  ) -> Option<bool> {
    dispatch_bail!(
      self,
      [
        opt import_meta,
        opt esm_import_dependency,
        opt common_js_imports,
        opt import_parser,
      ],
      can_collect_destructuring_assignment_properties(parser, expr)
    )
  }

  fn pattern(&self, parser: &mut JavascriptParser, ident: &Ident, for_name: &str) -> Option<bool> {
    dispatch_bail!(
      self,
      [req compatibility, opt worker],
      pattern(parser, ident, for_name)
    )
  }

  fn pre_declarator(
    &self,
    parser: &mut JavascriptParser,
    declarator: &VarDeclarator,
    declaration: VariableDeclaration<'_>,
  ) -> Option<bool> {
    dispatch_bail!(
      self,
      [
        req compatibility,
        opt common_js_imports,
        opt api,
        opt import_parser,
        opt worker,
        opt inline_const,
        opt inner_graph,
      ],
      pre_declarator(parser, declarator, declaration)
    )
  }

  fn can_rename(&self, parser: &mut JavascriptParser, str: &str) -> Option<bool> {
    dispatch_bail!(
      self,
      [opt amd_parser, opt common_js_imports],
      can_rename(parser, str)
    )
  }

  fn rename(&self, parser: &mut JavascriptParser, expr: &Expr, str: &str) -> Option<bool> {
    dispatch_bail!(
      self,
      [opt amd_parser, opt common_js_imports, opt node],
      rename(parser, expr, str)
    )
  }

  fn pre_statement(&self, parser: &mut JavascriptParser, stmt: Statement) -> Option<bool> {
    dispatch_bail!(
      self,
      [req compatibility, opt api, opt inner_graph],
      pre_statement(parser, stmt)
    )
  }

  fn block_pre_statement(&self, parser: &mut JavascriptParser, stmt: Statement) -> Option<bool> {
    dispatch_bail!(self, [opt inner_graph], block_pre_statement(parser, stmt))
  }

  fn import_call(
    &self,
    parser: &mut JavascriptParser,
    expr: &CallExpr,
    import_then: Option<&CallExpr>,
    members: Option<(&[Atom], bool)>,
  ) -> Option<bool> {
    assert!(expr.callee.is_import());
    dispatch_bail!(
      self,
      [opt import_parser],
      import_call(parser, expr, import_then, members)
    )
  }

  fn meta_property(
    &self,
    parser: &mut JavascriptParser,
    root_name: &Atom,
    span: Span,
  ) -> Option<bool> {
    dispatch_bail!(
      self,
      [opt import_meta, opt node],
      meta_property(parser, root_name, span)
    )
  }

  fn unhandled_expression_member_chain(
    &self,
    parser: &mut JavascriptParser,
    root_info: &ExportedVariableInfo,
    expr: &MemberExpr,
  ) -> Option<bool> {
    dispatch_bail!(
      self,
      [opt import_meta],
      unhandled_expression_member_chain(parser, root_info, expr)
    )
  }

  fn import(
    &self,
    parser: &mut JavascriptParser,
    statement: &ImportDecl,
    source: &str,
  ) -> Option<bool> {
    dispatch_bail!(
      self,
      [opt esm_import_dependency],
      import(parser, statement, source)
    )
  }

  fn import_specifier(
    &self,
    parser: &mut JavascriptParser,
    statement: &ImportDecl,
    source: &Atom,
    export_name: Option<&Atom>,
    identifier_name: &Atom,
  ) -> Option<bool> {
    dispatch_bail!(
      self,
      [opt esm_import_dependency],
      import_specifier(parser, statement, source, export_name, identifier_name)
    )
  }

  fn export_import(
    &self,
    parser: &mut JavascriptParser,
    statement: ExportImport,
    source: &Atom,
  ) -> Option<bool> {
    dispatch_bail!(
      self,
      [opt esm_export_dependency],
      export_import(parser, statement, source)
    )
  }

  fn export(&self, parser: &mut JavascriptParser, statement: ExportLocal) -> Option<bool> {
    dispatch_bail!(self, [opt esm_export_dependency], export(parser, statement))
  }

  fn export_import_specifier(
    &self,
    parser: &mut JavascriptParser,
    statement: ExportImport,
    source: &Atom,
    local_id: Option<&Atom>,
    export_name: Option<&Atom>,
    export_name_span: Option<Span>,
  ) -> Option<bool> {
    dispatch_bail!(
      self,
      [opt esm_export_dependency],
      export_import_specifier(
        parser,
        statement,
        source,
        local_id,
        export_name,
        export_name_span
      )
    )
  }

  fn export_specifier(
    &self,
    parser: &mut JavascriptParser,
    statement: ExportLocal,
    local_id: &Atom,
    export_name: &Atom,
    export_name_span: Span,
  ) -> Option<bool> {
    dispatch_bail!(
      self,
      [opt esm_export_dependency],
      export_specifier(parser, statement, local_id, export_name, export_name_span)
    )
  }

  fn export_expression(
    &self,
    parser: &mut JavascriptParser,
    statement: ExportDefaultDeclaration,
    expr: ExportDefaultExpression,
  ) -> Option<bool> {
    dispatch_bail!(
      self,
      [opt esm_export_dependency],
      export_expression(parser, statement, expr)
    )
  }

  fn expression_conditional_operation(
    &self,
    parser: &mut JavascriptParser,
    expr: &CondExpr,
  ) -> Option<bool> {
    dispatch_bail!(
      self,
      [req const_plugin],
      expression_conditional_operation(parser, expr)
    )
  }

  fn import_meta_property_in_destructuring(
    &self,
    parser: &mut JavascriptParser,
    property: &DestructuringAssignmentProperty,
  ) -> Option<String> {
    dispatch_bail!(
      self,
      [opt node],
      import_meta_property_in_destructuring(parser, property)
    )
  }
}
