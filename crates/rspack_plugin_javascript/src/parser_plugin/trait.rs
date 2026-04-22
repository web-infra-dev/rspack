use swc_core::{
  atoms::Atom,
  common::Span,
  ecma::ast::{
    AssignExpr, AwaitExpr, BinExpr, CallExpr, ClassMember, CondExpr, Expr, ForOfStmt, Ident,
    IfStmt, ImportDecl, MemberExpr, ModuleDecl, NewExpr, OptChainExpr, Program, ThisExpr,
    UnaryExpr, VarDeclarator,
  },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum JavascriptParserPluginHook {
  PreStatement,
  BlockPreStatement,
  TopLevelAwaitExpr,
  TopLevelForOfAwaitStmt,
  CanRename,
  Rename,
  Program,
  Statement,
  UnusedStatement,
  ModuleDeclaration,
  BlockPreModuleDeclaration,
  PreDeclarator,
  Evaluate,
  EvaluateTypeof,
  EvaluateCallExpression,
  EvaluateCallExpressionMember,
  EvaluateIdentifier,
  CanCollectDestructuringAssignmentProperties,
  Pattern,
  Call,
  CallMemberChain,
  Member,
  MemberChain,
  UnhandledExpressionMemberChain,
  MemberChainOfCallMemberChain,
  CallMemberChainOfCallMemberChain,
  Typeof,
  ExpressionLogicalOperator,
  BinaryExpression,
  StatementIf,
  ClassExtendsExpression,
  ClassBodyElement,
  ClassBodyValue,
  Declarator,
  NewExpression,
  Identifier,
  This,
  Assign,
  AssignMemberChain,
  ImportCall,
  MetaProperty,
  Import,
  ImportSpecifier,
  ExportImport,
  Export,
  ExportImportSpecifier,
  ExportSpecifier,
  ExportExpression,
  OptionalChaining,
  ExpressionConditionalOperation,
  Finish,
  IsPure,
  ImportMetaPropertyInDestructuring,
}

impl JavascriptParserPluginHook {
  pub const COUNT: usize = Self::ImportMetaPropertyInDestructuring as usize + 1;
  pub const ALL_MASK: u64 = if Self::COUNT == u64::BITS as usize {
    u64::MAX
  } else {
    (1u64 << Self::COUNT) - 1
  };

  pub const ALL: [Self; Self::COUNT] = [
    Self::PreStatement,
    Self::BlockPreStatement,
    Self::TopLevelAwaitExpr,
    Self::TopLevelForOfAwaitStmt,
    Self::CanRename,
    Self::Rename,
    Self::Program,
    Self::Statement,
    Self::UnusedStatement,
    Self::ModuleDeclaration,
    Self::BlockPreModuleDeclaration,
    Self::PreDeclarator,
    Self::Evaluate,
    Self::EvaluateTypeof,
    Self::EvaluateCallExpression,
    Self::EvaluateCallExpressionMember,
    Self::EvaluateIdentifier,
    Self::CanCollectDestructuringAssignmentProperties,
    Self::Pattern,
    Self::Call,
    Self::CallMemberChain,
    Self::Member,
    Self::MemberChain,
    Self::UnhandledExpressionMemberChain,
    Self::MemberChainOfCallMemberChain,
    Self::CallMemberChainOfCallMemberChain,
    Self::Typeof,
    Self::ExpressionLogicalOperator,
    Self::BinaryExpression,
    Self::StatementIf,
    Self::ClassExtendsExpression,
    Self::ClassBodyElement,
    Self::ClassBodyValue,
    Self::Declarator,
    Self::NewExpression,
    Self::Identifier,
    Self::This,
    Self::Assign,
    Self::AssignMemberChain,
    Self::ImportCall,
    Self::MetaProperty,
    Self::Import,
    Self::ImportSpecifier,
    Self::ExportImport,
    Self::Export,
    Self::ExportImportSpecifier,
    Self::ExportSpecifier,
    Self::ExportExpression,
    Self::OptionalChaining,
    Self::ExpressionConditionalOperation,
    Self::Finish,
    Self::IsPure,
    Self::ImportMetaPropertyInDestructuring,
  ];

  pub const fn mask(self) -> u64 {
    1u64 << (self as u8)
  }
}

const _: () = assert!(
  JavascriptParserPluginHook::COUNT <= 64,
  "The number of JavascriptParserPluginHook variants exceeds 64, which cannot be safely stored in a u64 bitmask."
);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JavascriptParserPluginHooks(u64);

impl JavascriptParserPluginHooks {
  pub const fn empty() -> Self {
    Self(0)
  }

  pub const fn all() -> Self {
    Self(JavascriptParserPluginHook::ALL_MASK)
  }

  pub const fn contains(self, hook: JavascriptParserPluginHook) -> bool {
    self.0 & hook.mask() != 0
  }

  pub const fn bits(self) -> u64 {
    self.0
  }

  pub const fn with(self, hook: JavascriptParserPluginHook) -> Self {
    Self(self.0 | hook.mask())
  }
}

use crate::{
  utils::eval::BasicEvaluatedExpression,
  visitors::{
    ClassDeclOrExpr, DestructuringAssignmentProperty, ExportDefaultDeclaration,
    ExportDefaultExpression, ExportImport, ExportLocal, ExportedVariableInfo, JavascriptParser,
    Statement, VariableDeclaration,
  },
};

type KeepRight = bool;

pub trait JavascriptParserPlugin {
  /// Used by the parser drive to precompute which hook paths this plugin
  /// actually implements.
  ///
  /// # Why this exists (performance)
  ///
  /// `JavaScriptParserPluginDrive` needs to call many hook methods while walking the AST.
  /// Calling every hook on every plugin would be very expensive.
  ///
  /// Instead, rspack precomputes a *hook mask* for each plugin at construction time:
  ///
  /// - Each plugin reports which hooks it actually implements via this method.
  /// - The drive groups plugins by hook up-front.
  /// - During parsing, the drive only iterates plugins that declared the current hook.
  ///
  /// This cuts a large amount of useless dynamic dispatch at runtime.
  ///
  /// # How to implement
  ///
  /// Do **NOT** implement this method manually.
  ///
  /// Use the proc-macro attribute `#[rspack_plugin_javascript::implemented_javascript_parser_hooks]`
  /// (or `#[rspack_macros::implemented_javascript_parser_hooks]` inside the workspace).
  /// The macro inspects the `impl JavascriptParserPlugin for ...` block and generates an
  /// efficient `implemented_hooks` implementation automatically.
  fn implemented_hooks(&self) -> JavascriptParserPluginHooks {
    // NOTE: The intended implementation is generated by the
    // `implemented_javascript_parser_hooks` attribute.
    // If we end up here, it usually means the attribute was forgotten.
    if cfg!(debug_assertions) {
      panic!(
        "`implemented_hooks` must be generated by the `implemented_javascript_parser_hooks` macro (attribute). \
Please annotate your `impl JavascriptParserPlugin for ...` block with `#[rspack_plugin_javascript::implemented_javascript_parser_hooks]` \
(or `#[rspack_macros::implemented_javascript_parser_hooks]` inside the repository)."
      );
    }

    // Production fallback: assume the plugin may implement any hook.
    JavascriptParserPluginHooks::all()
  }

  /// Return:
  /// - `Some(true)` signifies the termination of the current
  ///   statement's visit during the pre-walk phase.
  /// - Other return values imply that the walk operation ought to continue
  fn pre_statement(&self, _parser: &mut JavascriptParser, _stmt: Statement) -> Option<bool> {
    None
  }

  fn block_pre_statement(&self, _parser: &mut JavascriptParser, _stmt: Statement) -> Option<bool> {
    None
  }

  /// The return value will have no effect.
  fn top_level_await_expr(&self, _parser: &mut JavascriptParser, _expr: &AwaitExpr) {}

  /// The return value will have no effect.
  fn top_level_for_of_await_stmt(&self, _parser: &mut JavascriptParser, _stmt: &ForOfStmt) {}

  fn can_rename(&self, _parser: &mut JavascriptParser, _str: &str) -> Option<bool> {
    None
  }

  fn rename(&self, _parser: &mut JavascriptParser, _expr: &Expr, _str: &str) -> Option<bool> {
    None
  }

  fn program(&self, _parser: &mut JavascriptParser, _ast: &Program) -> Option<bool> {
    None
  }

  fn statement(&self, _parser: &mut JavascriptParser, _stmt: Statement) -> Option<bool> {
    None
  }

  /// Called for statements after a terminating point (when only function
  /// declarations should still be processed). Plugins may eliminate or
  /// transform such unused statements.
  ///
  /// Return:
  /// - `Some(true)` means the statement is fully handled and should be skipped
  /// - Other values mean the parser should still walk the statement
  fn unused_statement(&self, _parser: &mut JavascriptParser, _stmt: Statement) -> Option<bool> {
    None
  }

  fn module_declaration(&self, _parser: &mut JavascriptParser, _decl: &ModuleDecl) -> Option<bool> {
    None
  }

  /// Return:
  /// `None` means continue this `ModuleDecl`
  /// Others means skip this.
  ///
  /// This is similar `hooks.block_pre_statement` in webpack
  fn block_pre_module_declaration(
    &self,
    _parser: &mut JavascriptParser,
    _decl: &ModuleDecl,
  ) -> Option<bool> {
    None
  }

  fn pre_declarator(
    &self,
    _parser: &mut JavascriptParser,
    _declarator: &VarDeclarator,
    _declaration: VariableDeclaration<'_>,
  ) -> Option<bool> {
    None
  }

  fn evaluate<'a>(
    &self,
    _parser: &mut JavascriptParser,
    _expr: &'a Expr,
  ) -> Option<BasicEvaluatedExpression<'a>> {
    None
  }

  fn evaluate_typeof<'a>(
    &self,
    _parser: &mut JavascriptParser,
    _expr: &'a UnaryExpr,
    _for_name: &str,
  ) -> Option<BasicEvaluatedExpression<'a>> {
    None
  }

  fn evaluate_identifier(
    &self,
    _parser: &mut JavascriptParser,
    _for_name: &str,
    _start: u32,
    _end: u32,
  ) -> Option<BasicEvaluatedExpression<'static>> {
    None
  }

  /// Evaluate CallExpression when callee is an Identifier (e.g. String(), Number()).
  /// Mirrors webpack's hooks.evaluateCallExpression.
  fn evaluate_call_expression<'a>(
    &self,
    _parser: &mut JavascriptParser,
    _name: &str,
    _expr: &'a CallExpr,
  ) -> Option<BasicEvaluatedExpression<'a>> {
    None
  }

  fn evaluate_call_expression_member<'a>(
    &self,
    _parser: &mut JavascriptParser,
    _property: &str,
    _expr: &'a CallExpr,
    _param: BasicEvaluatedExpression<'a>,
  ) -> Option<BasicEvaluatedExpression<'a>> {
    None
  }

  fn can_collect_destructuring_assignment_properties(
    &self,
    _parser: &mut JavascriptParser,
    _expr: &Expr,
  ) -> Option<bool> {
    None
  }

  fn pattern(
    &self,
    _parser: &mut JavascriptParser,
    _ident: &Ident,
    _for_name: &str,
  ) -> Option<bool> {
    None
  }

  fn call(
    &self,
    _parser: &mut JavascriptParser,
    _expr: &CallExpr,
    _for_name: &str,
  ) -> Option<bool> {
    None
  }

  fn call_member_chain(
    &self,
    _parser: &mut JavascriptParser,
    _expr: &CallExpr,
    _for_name: &str,
    _members: &[Atom],
    _members_optionals: &[bool],
    _member_ranges: &[Span],
  ) -> Option<bool> {
    None
  }

  fn member(
    &self,
    _parser: &mut JavascriptParser,
    _expr: &MemberExpr,
    _for_name: &str,
  ) -> Option<bool> {
    None
  }

  fn member_chain(
    &self,
    _parser: &mut JavascriptParser,
    _expr: &MemberExpr,
    _for_name: &str,
    _members: &[Atom],
    _members_optionals: &[bool],
    _member_ranges: &[Span],
  ) -> Option<bool> {
    None
  }

  fn unhandled_expression_member_chain(
    &self,
    _parser: &mut JavascriptParser,
    _root_info: &ExportedVariableInfo,
    _expr: &MemberExpr,
  ) -> Option<bool> {
    None
  }

  #[allow(clippy::too_many_arguments)]
  fn member_chain_of_call_member_chain(
    &self,
    _parser: &mut JavascriptParser,
    _member_expr: &MemberExpr,
    _callee_members: &[Atom],
    _call_expr: &CallExpr,
    _members: &[Atom],
    _member_ranges: &[Span],
    _for_name: &str,
  ) -> Option<bool> {
    None
  }

  #[allow(clippy::too_many_arguments)]
  fn call_member_chain_of_call_member_chain(
    &self,
    _parser: &mut JavascriptParser,
    _call_expr: &CallExpr,
    _callee_members: &[Atom],
    _inner_call_expr: &CallExpr,
    _members: &[Atom],
    _member_ranges: &[Span],
    _for_name: &str,
  ) -> Option<bool> {
    None
  }

  fn r#typeof(
    &self,
    _parser: &mut JavascriptParser,
    _expr: &UnaryExpr,
    _for_name: &str,
  ) -> Option<bool> {
    None
  }

  /// Return:
  /// - `None` means should walk left and right;
  /// - `Some(true)` means should walk right;
  /// - `Some(false)` means nothing need to do.
  fn expression_logical_operator(
    &self,
    _parser: &mut JavascriptParser,
    _expr: &BinExpr,
  ) -> Option<KeepRight> {
    None
  }

  /// Return:
  /// - `None` means should walk left and right;
  fn binary_expression(&self, _parser: &mut JavascriptParser, _expr: &BinExpr) -> Option<bool> {
    None
  }

  /// Return:
  /// - `None` means need walk `stmt.test`, `stmt.cons` and `stmt.alt`;
  /// - `Some(true)` means only need walk `stmt.cons`;
  /// - `Some(false)` means only need walk `stmt.alt`;
  fn statement_if(&self, _parser: &mut JavascriptParser, _expr: &IfStmt) -> Option<bool> {
    None
  }

  fn class_extends_expression(
    &self,
    _parser: &mut JavascriptParser,
    _super_class: &Expr,
    _class_decl_or_expr: ClassDeclOrExpr,
  ) -> Option<bool> {
    None
  }

  fn class_body_element(
    &self,
    _parser: &mut JavascriptParser,
    _element: &ClassMember,
    _class_decl_or_expr: ClassDeclOrExpr,
  ) -> Option<bool> {
    None
  }

  fn class_body_value(
    &self,
    _parser: &mut JavascriptParser,
    _element: &swc_core::ecma::ast::ClassMember,
    _expr_span: Span,
    _class_decl_or_expr: ClassDeclOrExpr,
  ) -> Option<bool> {
    None
  }

  fn declarator(
    &self,
    _parser: &mut JavascriptParser,
    _expr: &VarDeclarator,
    _stmt: VariableDeclaration<'_>,
  ) -> Option<bool> {
    None
  }

  fn new_expression(
    &self,
    _parser: &mut JavascriptParser,
    _expr: &NewExpr,
    _for_name: &str,
  ) -> Option<bool> {
    None
  }

  fn identifier(
    &self,
    _parser: &mut JavascriptParser,
    _ident: &Ident,
    _for_name: &str,
  ) -> Option<bool> {
    None
  }

  fn this(
    &self,
    _parser: &mut JavascriptParser,
    _expr: &ThisExpr,
    _for_name: &str,
  ) -> Option<bool> {
    None
  }

  fn assign(
    &self,
    _parser: &mut JavascriptParser,
    _expr: &AssignExpr,
    _for_name: &str,
  ) -> Option<bool> {
    None
  }

  fn assign_member_chain(
    &self,
    _parser: &mut JavascriptParser,
    _expr: &AssignExpr,
    _members: &[Atom],
    _for_name: &str,
  ) -> Option<bool> {
    None
  }

  fn import_call(
    &self,
    _parser: &mut JavascriptParser,
    _expr: &CallExpr,
    _import_then: Option<&CallExpr>,
    _members: Option<(&[Atom], bool)>,
  ) -> Option<bool> {
    None
  }

  fn meta_property(
    &self,
    _parser: &mut JavascriptParser,
    _root_name: &Atom,
    _span: Span,
  ) -> Option<bool> {
    None
  }

  fn import(
    &self,
    _parser: &mut JavascriptParser,
    _statement: &ImportDecl,
    _source: &str,
  ) -> Option<bool> {
    None
  }

  fn import_specifier(
    &self,
    _parser: &mut JavascriptParser,
    _statement: &ImportDecl,
    _source: &Atom,
    _export_name: Option<&Atom>,
    _identifier_name: &Atom,
  ) -> Option<bool> {
    None
  }

  fn export_import(
    &self,
    _parser: &mut JavascriptParser,
    _statement: ExportImport,
    _source: &Atom,
  ) -> Option<bool> {
    None
  }

  fn export(&self, _parser: &mut JavascriptParser, _statement: ExportLocal) -> Option<bool> {
    None
  }

  fn export_import_specifier(
    &self,
    _parser: &mut JavascriptParser,
    _statement: ExportImport,
    _source: &Atom,
    _local_id: Option<&Atom>,
    _export_name: Option<&Atom>,
    _export_name_span: Option<Span>,
  ) -> Option<bool> {
    None
  }

  fn export_specifier(
    &self,
    _parser: &mut JavascriptParser,
    _statement: ExportLocal,
    _local_id: &Atom,
    _export_name: &Atom,
    _export_name_span: Span,
  ) -> Option<bool> {
    None
  }

  fn export_expression(
    &self,
    _parser: &mut JavascriptParser,
    _statement: ExportDefaultDeclaration,
    _expr: ExportDefaultExpression,
  ) -> Option<bool> {
    None
  }

  fn optional_chaining(
    &self,
    _parser: &mut JavascriptParser,
    _expr: &OptChainExpr,
  ) -> Option<bool> {
    None
  }

  fn expression_conditional_operation(
    &self,
    _parser: &mut JavascriptParser,
    _expr: &CondExpr,
  ) -> Option<bool> {
    None
  }

  fn finish(&self, _parser: &mut JavascriptParser) -> Option<bool> {
    None
  }

  fn is_pure(&self, _parser: &mut JavascriptParser, _expr: &Expr) -> Option<bool> {
    None
  }

  /* plugin interop methods */

  /**
   * This method is used to interop with other plugins.
   * It will be called in ImportMetaPlugin when processing destructuring of `import.meta`
   */
  fn import_meta_property_in_destructuring(
    &self,
    _parser: &mut JavascriptParser,
    _property: &DestructuringAssignmentProperty,
  ) -> Option<String> {
    None
  }
}

pub type BoxJavascriptParserPlugin = Box<dyn JavascriptParserPlugin + Send + Sync>;
