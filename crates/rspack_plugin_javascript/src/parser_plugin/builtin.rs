use rspack_core::{CompilerOptions, JavascriptParserOptions};
use swc_core::{
  atoms::Atom,
  common::Span,
  ecma::ast::{
    AssignExpr, AwaitExpr, BinExpr, CallExpr, Callee, CondExpr, Expr, ForOfStmt, Ident, IfStmt,
    ImportDecl, MemberExpr, ModuleDecl, NewExpr, Program, ThisExpr, UnaryExpr, UnaryOp,
    VarDeclarator,
  },
};

use super::{
  AMDDefineDependencyParserPlugin, AMDParserPlugin, AMDRequireDependenciesBlockParserPlugin,
  APIPlugin, CheckVarDeclaratorIdent, CommonJsImportsParserPlugin, CommonJsPlugin,
  CompatibilityPlugin, ConstPlugin, ESMDetectionParserPlugin, ESMExportDependencyParserPlugin,
  ESMImportDependencyParserPlugin, ESMTopLevelThisParserPlugin, ExportsInfoApiPlugin,
  ImportMetaContextDependencyParserPlugin, ImportParserPlugin, InitializeEvaluating,
  IsIncludedPlugin, JavascriptMetaInfoPlugin, JavascriptParserPlugin, JavascriptParserPluginHooks,
  OverrideStrictPlugin, RequireContextDependencyParserPlugin,
  RequireEnsureDependenciesBlockParserPlugin, UseStrictPlugin, WorkerPlugin, r#const::is_logic_op,
};
use crate::{
  utils::eval::BasicEvaluatedExpression,
  visitors::{
    ExportDefaultDeclaration, ExportDefaultExpression, ExportImport, ExportLocal, JavascriptParser,
    Statement, VariableDeclaration,
  },
};

macro_rules! dispatch_void_plugin {
  ($self:expr, $plugin:ident, $method:ident($($arg:expr),* $(,)?)) => {{
    JavascriptParserPlugin::$method(&$self.$plugin, $($arg),*);
  }};
}

macro_rules! dispatch_bail_plugin {
  ($self:expr, $plugin:ident, $method:ident($($arg:expr),* $(,)?)) => {{
    let res = JavascriptParserPlugin::$method(&$self.$plugin, $($arg),*);
    if res.is_some() {
      return res;
    }
  }};
}

macro_rules! dispatch_eval_bail_plugin {
  ($self:expr, $plugin:ident, $method:ident($($arg:expr),* $(,)?), $param:expr) => {{
    let res = JavascriptParserPlugin::$method(&$self.$plugin, $($arg),*, $param.clone());
    if res.is_some() {
      return res;
    }
  }};
}

macro_rules! dispatch_void {
  ($self:expr, [$($plugin:ident),* $(,)?], $method:ident $args:tt) => {{
    $(dispatch_void_plugin!($self, $plugin, $method $args);)*
  }};
}

macro_rules! dispatch_bail {
  ($self:expr, [$($plugin:ident),* $(,)?], $method:ident $args:tt) => {{
    $(dispatch_bail_plugin!($self, $plugin, $method $args);)*
    None
  }};
}

macro_rules! dispatch_eval_bail {
  ($self:expr, [$($plugin:ident),* $(,)?], $method:ident $args:tt, $param:expr) => {{
    $(dispatch_eval_bail_plugin!($self, $plugin, $method $args, $param);)*
    None
  }};
}

fn union_hook_sets(
  hook_sets: impl IntoIterator<Item = JavascriptParserPluginHooks>,
) -> JavascriptParserPluginHooks {
  hook_sets
    .into_iter()
    .fold(JavascriptParserPluginHooks::empty(), |acc, hooks| {
      acc.union(hooks)
    })
}

pub(crate) struct BaseBuiltinJavascriptParserPlugin {
  initialize_evaluating: InitializeEvaluating,
  javascript_meta_info: JavascriptMetaInfoPlugin,
  check_var_declarator_ident: CheckVarDeclaratorIdent,
  const_plugin: ConstPlugin,
  use_strict: UseStrictPlugin,
  require_context_dependency: RequireContextDependencyParserPlugin,
  require_ensure_dependencies_block: RequireEnsureDependenciesBlockParserPlugin,
  compatibility: CompatibilityPlugin,
}

impl BaseBuiltinJavascriptParserPlugin {
  pub(crate) fn new() -> Self {
    Self {
      initialize_evaluating: InitializeEvaluating,
      javascript_meta_info: JavascriptMetaInfoPlugin,
      check_var_declarator_ident: CheckVarDeclaratorIdent,
      const_plugin: ConstPlugin,
      use_strict: UseStrictPlugin,
      require_context_dependency: RequireContextDependencyParserPlugin,
      require_ensure_dependencies_block: RequireEnsureDependenciesBlockParserPlugin,
      compatibility: CompatibilityPlugin,
    }
  }
}

impl JavascriptParserPlugin for BaseBuiltinJavascriptParserPlugin {
  fn implemented_hooks(&self) -> JavascriptParserPluginHooks {
    union_hook_sets([
      self.initialize_evaluating.implemented_hooks(),
      self.javascript_meta_info.implemented_hooks(),
      self.check_var_declarator_ident.implemented_hooks(),
      self.const_plugin.implemented_hooks(),
      self.use_strict.implemented_hooks(),
      self.require_context_dependency.implemented_hooks(),
      self.require_ensure_dependencies_block.implemented_hooks(),
      self.compatibility.implemented_hooks(),
    ])
  }

  fn program(&self, parser: &mut JavascriptParser, ast: &Program) -> Option<bool> {
    dispatch_bail!(self, [use_strict, compatibility], program(parser, ast))
  }

  fn finish(&self, parser: &mut JavascriptParser) -> Option<bool> {
    dispatch_bail!(self, [javascript_meta_info], finish(parser))
  }

  fn call(&self, parser: &mut JavascriptParser, expr: &CallExpr, name: &str) -> Option<bool> {
    dispatch_bail!(
      self,
      [
        javascript_meta_info,
        require_context_dependency,
        require_ensure_dependencies_block,
        compatibility,
      ],
      call(parser, expr, name)
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
      [require_ensure_dependencies_block],
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
      [const_plugin],
      expression_logical_operator(parser, expr)
    )
  }

  fn unused_statement(&self, parser: &mut JavascriptParser, stmt: Statement) -> Option<bool> {
    dispatch_bail!(self, [const_plugin], unused_statement(parser, stmt))
  }

  fn statement_if(&self, parser: &mut JavascriptParser, expr: &IfStmt) -> Option<bool> {
    dispatch_bail!(self, [const_plugin], statement_if(parser, expr))
  }

  fn declarator(
    &self,
    parser: &mut JavascriptParser,
    expr: &VarDeclarator,
    stmt: VariableDeclaration<'_>,
  ) -> Option<bool> {
    dispatch_bail!(
      self,
      [check_var_declarator_ident],
      declarator(parser, expr, stmt)
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
      [const_plugin, compatibility],
      identifier(parser, expr, for_name)
    )
  }

  fn evaluate_typeof<'a>(
    &self,
    parser: &mut JavascriptParser,
    expr: &'a UnaryExpr,
    for_name: &str,
  ) -> Option<BasicEvaluatedExpression<'a>> {
    dispatch_bail!(
      self,
      [require_ensure_dependencies_block],
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
      [initialize_evaluating],
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
      [initialize_evaluating],
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
      [const_plugin],
      evaluate_identifier(parser, for_name, start, end)
    )
  }

  fn pattern(&self, parser: &mut JavascriptParser, ident: &Ident, for_name: &str) -> Option<bool> {
    dispatch_bail!(self, [compatibility], pattern(parser, ident, for_name))
  }

  fn pre_declarator(
    &self,
    parser: &mut JavascriptParser,
    declarator: &VarDeclarator,
    declaration: VariableDeclaration<'_>,
  ) -> Option<bool> {
    dispatch_bail!(
      self,
      [compatibility],
      pre_declarator(parser, declarator, declaration)
    )
  }

  fn pre_statement(&self, parser: &mut JavascriptParser, stmt: Statement) -> Option<bool> {
    dispatch_bail!(self, [compatibility], pre_statement(parser, stmt))
  }

  fn expression_conditional_operation(
    &self,
    parser: &mut JavascriptParser,
    expr: &CondExpr,
  ) -> Option<bool> {
    dispatch_bail!(
      self,
      [const_plugin],
      expression_conditional_operation(parser, expr)
    )
  }
}

pub(crate) struct EsmBuiltinJavascriptParserPlugin {
  esm_top_level_this: ESMTopLevelThisParserPlugin,
  esm_detection: ESMDetectionParserPlugin,
  import_meta_context_dependency: ImportMetaContextDependencyParserPlugin,
  esm_import_dependency: ESMImportDependencyParserPlugin,
  esm_export_dependency: ESMExportDependencyParserPlugin,
}

impl EsmBuiltinJavascriptParserPlugin {
  pub(crate) fn new() -> Self {
    Self {
      esm_top_level_this: ESMTopLevelThisParserPlugin,
      esm_detection: ESMDetectionParserPlugin,
      import_meta_context_dependency: ImportMetaContextDependencyParserPlugin,
      esm_import_dependency: ESMImportDependencyParserPlugin,
      esm_export_dependency: ESMExportDependencyParserPlugin,
    }
  }
}

impl JavascriptParserPlugin for EsmBuiltinJavascriptParserPlugin {
  fn implemented_hooks(&self) -> JavascriptParserPluginHooks {
    union_hook_sets([
      self.esm_top_level_this.implemented_hooks(),
      self.esm_detection.implemented_hooks(),
      self.import_meta_context_dependency.implemented_hooks(),
      self.esm_import_dependency.implemented_hooks(),
      self.esm_export_dependency.implemented_hooks(),
    ])
  }

  fn top_level_await_expr(&self, parser: &mut JavascriptParser, expr: &AwaitExpr) {
    dispatch_void!(self, [esm_detection], top_level_await_expr(parser, expr));
  }

  fn top_level_for_of_await_stmt(&self, parser: &mut JavascriptParser, stmt: &ForOfStmt) {
    dispatch_void!(
      self,
      [esm_detection],
      top_level_for_of_await_stmt(parser, stmt)
    );
  }

  fn program(&self, parser: &mut JavascriptParser, ast: &Program) -> Option<bool> {
    dispatch_bail!(self, [esm_detection], program(parser, ast))
  }

  fn module_declaration(&self, parser: &mut JavascriptParser, decl: &ModuleDecl) -> Option<bool> {
    dispatch_bail!(
      self,
      [esm_export_dependency],
      module_declaration(parser, decl)
    )
  }

  fn call(&self, parser: &mut JavascriptParser, expr: &CallExpr, name: &str) -> Option<bool> {
    dispatch_bail!(
      self,
      [esm_detection, import_meta_context_dependency],
      call(parser, expr, name)
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
      [esm_import_dependency],
      member_chain(
        parser,
        expr,
        for_name,
        members,
        members_optionals,
        member_ranges
      )
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
      [esm_import_dependency],
      call_member_chain(
        parser,
        expr,
        for_name,
        members,
        members_optionals,
        member_ranges
      )
    )
  }

  fn r#typeof(
    &self,
    parser: &mut JavascriptParser,
    expr: &UnaryExpr,
    for_name: &str,
  ) -> Option<bool> {
    assert!(expr.op == UnaryOp::TypeOf);
    dispatch_bail!(self, [esm_detection], r#typeof(parser, expr, for_name))
  }

  fn binary_expression(&self, parser: &mut JavascriptParser, expr: &BinExpr) -> Option<bool> {
    assert!(!is_logic_op(expr.op));
    dispatch_bail!(
      self,
      [esm_import_dependency],
      binary_expression(parser, expr)
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
      [esm_detection, esm_import_dependency],
      identifier(parser, expr, for_name)
    )
  }

  fn this(&self, parser: &mut JavascriptParser, expr: &ThisExpr, for_name: &str) -> Option<bool> {
    dispatch_bail!(self, [esm_top_level_this], this(parser, expr, for_name))
  }

  fn evaluate_typeof<'a>(
    &self,
    parser: &mut JavascriptParser,
    expr: &'a UnaryExpr,
    for_name: &str,
  ) -> Option<BasicEvaluatedExpression<'a>> {
    dispatch_bail!(
      self,
      [esm_detection],
      evaluate_typeof(parser, expr, for_name)
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
      [import_meta_context_dependency],
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
      [esm_import_dependency],
      can_collect_destructuring_assignment_properties(parser, expr)
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
      [esm_import_dependency],
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
      [esm_import_dependency],
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
      [esm_export_dependency],
      export_import(parser, statement, source)
    )
  }

  fn export(&self, parser: &mut JavascriptParser, statement: ExportLocal) -> Option<bool> {
    dispatch_bail!(self, [esm_export_dependency], export(parser, statement))
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
      [esm_export_dependency],
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
      [esm_export_dependency],
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
      [esm_export_dependency],
      export_expression(parser, statement, expr)
    )
  }
}

pub(crate) struct CommonJsBuiltinJavascriptParserPlugin {
  common_js_imports: CommonJsImportsParserPlugin,
  common_js: CommonJsPlugin,
}

impl CommonJsBuiltinJavascriptParserPlugin {
  pub(crate) fn new() -> Self {
    Self {
      common_js_imports: CommonJsImportsParserPlugin,
      common_js: CommonJsPlugin,
    }
  }
}

impl JavascriptParserPlugin for CommonJsBuiltinJavascriptParserPlugin {
  fn implemented_hooks(&self) -> JavascriptParserPluginHooks {
    union_hook_sets([
      self.common_js_imports.implemented_hooks(),
      self.common_js.implemented_hooks(),
    ])
  }

  fn finish(&self, parser: &mut JavascriptParser) -> Option<bool> {
    dispatch_bail!(self, [common_js_imports], finish(parser))
  }

  fn call(&self, parser: &mut JavascriptParser, expr: &CallExpr, name: &str) -> Option<bool> {
    dispatch_bail!(self, [common_js_imports], call(parser, expr, name))
  }

  fn member(
    &self,
    parser: &mut JavascriptParser,
    expr: &MemberExpr,
    for_name: &str,
  ) -> Option<bool> {
    dispatch_bail!(self, [common_js], member(parser, expr, for_name))
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
      [common_js_imports],
      member_chain(
        parser,
        expr,
        for_name,
        members,
        members_optionals,
        member_ranges
      )
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
      [common_js_imports],
      call_member_chain(
        parser,
        expr,
        for_name,
        members,
        members_optionals,
        member_ranges
      )
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
      [common_js_imports],
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
      [common_js_imports],
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
    dispatch_bail!(self, [common_js_imports], assign(parser, expr, for_name))
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
      [common_js_imports, common_js],
      r#typeof(parser, expr, for_name)
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
      [common_js_imports],
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
      [common_js_imports],
      identifier(parser, expr, for_name)
    )
  }

  fn evaluate_typeof<'a>(
    &self,
    parser: &mut JavascriptParser,
    expr: &'a UnaryExpr,
    for_name: &str,
  ) -> Option<BasicEvaluatedExpression<'a>> {
    dispatch_bail!(
      self,
      [common_js_imports],
      evaluate_typeof(parser, expr, for_name)
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
      [common_js_imports, common_js],
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
      [common_js_imports],
      can_collect_destructuring_assignment_properties(parser, expr)
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
      [common_js_imports],
      pre_declarator(parser, declarator, declaration)
    )
  }

  fn can_rename(&self, parser: &mut JavascriptParser, str: &str) -> Option<bool> {
    dispatch_bail!(self, [common_js_imports], can_rename(parser, str))
  }

  fn rename(&self, parser: &mut JavascriptParser, expr: &Expr, str: &str) -> Option<bool> {
    dispatch_bail!(self, [common_js_imports], rename(parser, expr, str))
  }
}

pub(crate) struct AmdBuiltinJavascriptParserPlugin {
  amd_require_dependencies_block: AMDRequireDependenciesBlockParserPlugin,
  amd_define_dependency: AMDDefineDependencyParserPlugin,
  amd_parser: AMDParserPlugin,
}

impl AmdBuiltinJavascriptParserPlugin {
  pub(crate) fn new() -> Self {
    Self {
      amd_require_dependencies_block: AMDRequireDependenciesBlockParserPlugin,
      amd_define_dependency: AMDDefineDependencyParserPlugin,
      amd_parser: AMDParserPlugin,
    }
  }
}

impl JavascriptParserPlugin for AmdBuiltinJavascriptParserPlugin {
  fn implemented_hooks(&self) -> JavascriptParserPluginHooks {
    union_hook_sets([
      self.amd_require_dependencies_block.implemented_hooks(),
      self.amd_define_dependency.implemented_hooks(),
      self.amd_parser.implemented_hooks(),
    ])
  }

  fn finish(&self, parser: &mut JavascriptParser) -> Option<bool> {
    dispatch_bail!(self, [amd_define_dependency], finish(parser))
  }

  fn call(&self, parser: &mut JavascriptParser, expr: &CallExpr, name: &str) -> Option<bool> {
    dispatch_bail!(
      self,
      [
        amd_require_dependencies_block,
        amd_define_dependency,
        amd_parser
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
    dispatch_bail!(self, [amd_parser], member(parser, expr, for_name))
  }

  fn r#typeof(
    &self,
    parser: &mut JavascriptParser,
    expr: &UnaryExpr,
    for_name: &str,
  ) -> Option<bool> {
    assert!(expr.op == UnaryOp::TypeOf);
    dispatch_bail!(self, [amd_parser], r#typeof(parser, expr, for_name))
  }

  fn identifier(
    &self,
    parser: &mut JavascriptParser,
    expr: &Ident,
    for_name: &str,
  ) -> Option<bool> {
    dispatch_bail!(self, [amd_parser], identifier(parser, expr, for_name))
  }

  fn evaluate_typeof<'a>(
    &self,
    parser: &mut JavascriptParser,
    expr: &'a UnaryExpr,
    for_name: &str,
  ) -> Option<BasicEvaluatedExpression<'a>> {
    dispatch_bail!(self, [amd_parser], evaluate_typeof(parser, expr, for_name))
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
      [amd_parser],
      evaluate_identifier(parser, for_name, start, end)
    )
  }

  fn can_rename(&self, parser: &mut JavascriptParser, str: &str) -> Option<bool> {
    dispatch_bail!(self, [amd_parser], can_rename(parser, str))
  }

  fn rename(&self, parser: &mut JavascriptParser, expr: &Expr, str: &str) -> Option<bool> {
    dispatch_bail!(self, [amd_parser], rename(parser, expr, str))
  }
}

pub(crate) struct JavascriptModuleBuiltinJavascriptParserPlugin {
  is_included: IsIncludedPlugin,
  exports_info_api: ExportsInfoApiPlugin,
  api: APIPlugin,
  import_parser: ImportParserPlugin,
  worker: WorkerPlugin,
  override_strict: OverrideStrictPlugin,
}

impl JavascriptModuleBuiltinJavascriptParserPlugin {
  pub(crate) fn new(
    compiler_options: &CompilerOptions,
    javascript_options: &JavascriptParserOptions,
  ) -> Self {
    Self {
      is_included: IsIncludedPlugin,
      exports_info_api: ExportsInfoApiPlugin,
      api: APIPlugin::new(compiler_options.output.module),
      import_parser: ImportParserPlugin,
      worker: WorkerPlugin::new(
        javascript_options
          .worker
          .as_ref()
          .expect("should have worker"),
      ),
      override_strict: OverrideStrictPlugin,
    }
  }
}

impl JavascriptParserPlugin for JavascriptModuleBuiltinJavascriptParserPlugin {
  fn implemented_hooks(&self) -> JavascriptParserPluginHooks {
    union_hook_sets([
      self.is_included.implemented_hooks(),
      self.exports_info_api.implemented_hooks(),
      self.api.implemented_hooks(),
      self.import_parser.implemented_hooks(),
      self.worker.implemented_hooks(),
      self.override_strict.implemented_hooks(),
    ])
  }

  fn program(&self, parser: &mut JavascriptParser, ast: &Program) -> Option<bool> {
    dispatch_bail!(self, [override_strict], program(parser, ast))
  }

  fn finish(&self, parser: &mut JavascriptParser) -> Option<bool> {
    dispatch_bail!(self, [import_parser], finish(parser))
  }

  fn call(&self, parser: &mut JavascriptParser, expr: &CallExpr, name: &str) -> Option<bool> {
    dispatch_bail!(self, [is_included, api, worker], call(parser, expr, name))
  }

  fn member(
    &self,
    parser: &mut JavascriptParser,
    expr: &MemberExpr,
    for_name: &str,
  ) -> Option<bool> {
    dispatch_bail!(self, [api], member(parser, expr, for_name))
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
      [exports_info_api, import_parser],
      member_chain(
        parser,
        expr,
        for_name,
        members,
        members_optionals,
        member_ranges
      )
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
      [import_parser, worker],
      call_member_chain(
        parser,
        expr,
        for_name,
        members,
        members_optionals,
        member_ranges
      )
    )
  }

  fn r#typeof(
    &self,
    parser: &mut JavascriptParser,
    expr: &UnaryExpr,
    for_name: &str,
  ) -> Option<bool> {
    assert!(expr.op == UnaryOp::TypeOf);
    dispatch_bail!(self, [is_included], r#typeof(parser, expr, for_name))
  }

  fn identifier(
    &self,
    parser: &mut JavascriptParser,
    expr: &Ident,
    for_name: &str,
  ) -> Option<bool> {
    dispatch_bail!(
      self,
      [exports_info_api, api, import_parser],
      identifier(parser, expr, for_name)
    )
  }

  fn new_expression(
    &self,
    parser: &mut JavascriptParser,
    expr: &NewExpr,
    for_name: &str,
  ) -> Option<bool> {
    dispatch_bail!(self, [worker], new_expression(parser, expr, for_name))
  }

  fn evaluate_typeof<'a>(
    &self,
    parser: &mut JavascriptParser,
    expr: &'a UnaryExpr,
    for_name: &str,
  ) -> Option<BasicEvaluatedExpression<'a>> {
    dispatch_bail!(self, [api], evaluate_typeof(parser, expr, for_name))
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
      [api],
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
      [import_parser],
      can_collect_destructuring_assignment_properties(parser, expr)
    )
  }

  fn pattern(&self, parser: &mut JavascriptParser, ident: &Ident, for_name: &str) -> Option<bool> {
    dispatch_bail!(self, [worker], pattern(parser, ident, for_name))
  }

  fn pre_declarator(
    &self,
    parser: &mut JavascriptParser,
    declarator: &VarDeclarator,
    declaration: VariableDeclaration<'_>,
  ) -> Option<bool> {
    dispatch_bail!(
      self,
      [api, import_parser, worker],
      pre_declarator(parser, declarator, declaration)
    )
  }

  fn pre_statement(&self, parser: &mut JavascriptParser, stmt: Statement) -> Option<bool> {
    dispatch_bail!(self, [api], pre_statement(parser, stmt))
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
      [import_parser],
      import_call(parser, expr, import_then, members)
    )
  }
}
