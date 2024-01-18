use std::sync::Arc;

use rspack_core::{
  BuildInfo, ConstDependency, Dependency, DependencyLocation, DependencyTemplate, ResourceData,
  RuntimeGlobals, RuntimeRequirementsDependency, SpanExt,
};
use rspack_error::miette::Diagnostic;
use rustc_hash::FxHashSet;
use swc_core::ecma::visit::{noop_visit_type, Visit, VisitWith};
use swc_core::{
  common::{SourceFile, Spanned, SyntaxContext},
  ecma::ast::{AssignExpr, AssignOp, CallExpr, Callee, Expr, Ident, Pat, PatOrExpr, VarDeclarator},
};

use super::{expr_matcher, JavascriptParser};
use crate::dependency::ModuleArgumentDependency;
use crate::no_visit_ignored_stmt;
use crate::parser_plugin::JavascriptParserPlugin;
use crate::utils::eval::{self, BasicEvaluatedExpression};
use crate::visitors::extract_member_root;

pub const WEBPACK_HASH: &str = "__webpack_hash__";
pub const WEBPACK_PUBLIC_PATH: &str = "__webpack_public_path__";
pub const WEBPACK_MODULES: &str = "__webpack_modules__";
pub const WEBPACK_MODULE: &str = "__webpack_module__";
pub const WEBPACK_RESOURCE_QUERY: &str = "__resourceQuery";
pub const WEBPACK_CHUNK_LOAD: &str = "__webpack_chunk_load__";
pub const WEBPACK_BASE_URI: &str = "__webpack_base_uri__";
pub const NON_WEBPACK_REQUIRE: &str = "__non_webpack_require__";
pub const SYSTEM_CONTEXT: &str = "__system_context__";
pub const WEBPACK_SHARE_SCOPES: &str = "__webpack_share_scopes__";
pub const WEBPACK_INIT_SHARING: &str = "__webpack_init_sharing__";
pub const WEBPACK_NONCE: &str = "__webpack_nonce__";
pub const WEBPACK_CHUNK_NAME: &str = "__webpack_chunkname__";
pub const WEBPACK_RUNTIME_ID: &str = "__webpack_runtime_id__";
pub const WEBPACK_REQUIRE: &str = "__webpack_require__";
pub const RSPACK_VERSION: &str = "__rspack_version__";

pub fn get_typeof_evaluate_of_api(sym: &str) -> Option<&str> {
  match sym {
    WEBPACK_REQUIRE => Some("function"),
    WEBPACK_HASH => Some("string"),
    WEBPACK_PUBLIC_PATH => Some("string"),
    WEBPACK_MODULES => Some("object"),
    WEBPACK_MODULE => Some("object"),
    WEBPACK_RESOURCE_QUERY => Some("string"),
    WEBPACK_CHUNK_LOAD => Some("function"),
    WEBPACK_BASE_URI => Some("string"),
    NON_WEBPACK_REQUIRE => None,
    SYSTEM_CONTEXT => Some("object"),
    WEBPACK_SHARE_SCOPES => Some("object"),
    WEBPACK_INIT_SHARING => Some("function"),
    WEBPACK_NONCE => Some("string"),
    WEBPACK_CHUNK_NAME => Some("string"),
    WEBPACK_RUNTIME_ID => None,
    RSPACK_VERSION => Some("string"),
    _ => None,
  }
}

pub struct ApiParserPlugin;

impl JavascriptParserPlugin for ApiParserPlugin {
  fn evaluate_typeof(
    &self,
    parser: &mut JavascriptParser,
    expression: &Ident,
    start: u32,
    end: u32,
  ) -> Option<BasicEvaluatedExpression> {
    if parser.is_unresolved_ident(expression.sym.as_str()) {
      get_typeof_evaluate_of_api(expression.sym.as_str())
        .map(|res| eval::evaluate_to_string(res.to_string(), start, end))
    } else {
      None
    }
  }
}

pub struct ApiScanner<'a> {
  pub source_file: Arc<SourceFile>,
  pub unresolved_ctxt: SyntaxContext,
  pub module: bool,
  pub build_info: &'a mut BuildInfo,
  pub enter_assign: bool,
  pub resource_data: &'a ResourceData,
  pub presentational_dependencies: &'a mut Vec<Box<dyn DependencyTemplate>>,
  pub dependencies: &'a mut Vec<Box<dyn Dependency>>,
  pub warning_diagnostics: &'a mut Vec<Box<dyn Diagnostic + Send + Sync>>,
  pub ignored: &'a mut FxHashSet<DependencyLocation>,
}

impl<'a> ApiScanner<'a> {
  #[allow(clippy::too_many_arguments)]
  pub fn new(
    source_file: Arc<SourceFile>,
    unresolved_ctxt: SyntaxContext,
    resource_data: &'a ResourceData,
    dependencies: &'a mut Vec<Box<dyn Dependency>>,
    presentational_dependencies: &'a mut Vec<Box<dyn DependencyTemplate>>,
    module: bool,
    build_info: &'a mut BuildInfo,
    warning_diagnostics: &'a mut Vec<Box<dyn Diagnostic + Send + Sync>>,
    ignored: &'a mut FxHashSet<DependencyLocation>,
  ) -> Self {
    Self {
      source_file,
      unresolved_ctxt,
      module,
      build_info,
      enter_assign: false,
      resource_data,
      presentational_dependencies,
      dependencies,
      warning_diagnostics,
      ignored,
    }
  }
}

impl Visit for ApiScanner<'_> {
  noop_visit_type!();
  no_visit_ignored_stmt!();

  fn visit_var_declarator(&mut self, var_declarator: &VarDeclarator) {
    match &var_declarator.name {
      Pat::Ident(ident) => {
        self.enter_assign = true;
        ident.visit_children_with(self);
        self.enter_assign = false;
      }
      _ => var_declarator.name.visit_children_with(self),
    }
    var_declarator.init.visit_children_with(self);
  }

  fn visit_assign_expr(&mut self, assign_expr: &AssignExpr) {
    if matches!(assign_expr.op, AssignOp::Assign) {
      match &assign_expr.left {
        PatOrExpr::Pat(box Pat::Ident(ident)) => {
          self.enter_assign = true;
          ident.visit_children_with(self);
          self.enter_assign = false;
        }
        _ => assign_expr.left.visit_children_with(self),
      }
    }
    assign_expr.right.visit_children_with(self);
  }

  fn visit_ident(&mut self, ident: &Ident) {
    if ident.span.ctxt != self.unresolved_ctxt {
      return;
    }
    match ident.sym.as_ref() as &str {
      WEBPACK_REQUIRE => {
        self
          .presentational_dependencies
          .push(Box::new(ConstDependency::new(
            ident.span.real_lo(),
            ident.span.real_hi(),
            RuntimeGlobals::REQUIRE.name().into(),
            Some(RuntimeGlobals::REQUIRE),
          )));
      }
      WEBPACK_HASH => {
        self
          .presentational_dependencies
          .push(Box::new(ConstDependency::new(
            ident.span.real_lo(),
            ident.span.real_hi(),
            format!("{}()", RuntimeGlobals::GET_FULL_HASH).into(),
            Some(RuntimeGlobals::GET_FULL_HASH),
          )));
      }
      WEBPACK_PUBLIC_PATH => {
        self
          .presentational_dependencies
          .push(Box::new(ConstDependency::new(
            ident.span.real_lo(),
            ident.span.real_hi(),
            RuntimeGlobals::PUBLIC_PATH.name().into(),
            Some(RuntimeGlobals::PUBLIC_PATH),
          )));
      }
      WEBPACK_MODULES => {
        if self.enter_assign {
          return;
        }
        self
          .presentational_dependencies
          .push(Box::new(ConstDependency::new(
            ident.span.real_lo(),
            ident.span.real_hi(),
            RuntimeGlobals::MODULE_FACTORIES.name().into(),
            Some(RuntimeGlobals::MODULE_FACTORIES),
          )));
      }
      WEBPACK_RESOURCE_QUERY => {
        let resource_query = self.resource_data.resource_query.as_deref().unwrap_or("");
        self
          .presentational_dependencies
          .push(Box::new(ConstDependency::new(
            ident.span.real_lo(),
            ident.span.real_hi(),
            serde_json::to_string(resource_query)
              .expect("should render module id")
              .into(),
            None,
          )));
      }
      WEBPACK_CHUNK_LOAD => {
        self
          .presentational_dependencies
          .push(Box::new(ConstDependency::new(
            ident.span.real_lo(),
            ident.span.real_hi(),
            RuntimeGlobals::ENSURE_CHUNK.name().into(),
            Some(RuntimeGlobals::ENSURE_CHUNK),
          )));
      }
      WEBPACK_MODULE => {
        self
          .presentational_dependencies
          .push(Box::new(ModuleArgumentDependency::new(
            ident.span.real_lo(),
            ident.span.real_hi(),
            None,
          )));
      }
      WEBPACK_BASE_URI => {
        self
          .presentational_dependencies
          .push(Box::new(ConstDependency::new(
            ident.span.real_lo(),
            ident.span.real_hi(),
            RuntimeGlobals::BASE_URI.name().into(),
            Some(RuntimeGlobals::BASE_URI),
          )));
      }
      NON_WEBPACK_REQUIRE => {
        self
          .presentational_dependencies
          .push(Box::new(ConstDependency::new(
            ident.span.real_lo(),
            ident.span.real_hi(),
            if self.module {
              self.build_info.need_create_require = true;
              "__WEBPACK_EXTERNAL_createRequire(import.meta.url)".into()
            } else {
              "require".into()
            },
            None,
          )));
      }
      SYSTEM_CONTEXT => self
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          ident.span.real_lo(),
          ident.span.real_hi(),
          RuntimeGlobals::SYSTEM_CONTEXT.name().into(),
          Some(RuntimeGlobals::SYSTEM_CONTEXT),
        ))),
      WEBPACK_SHARE_SCOPES => {
        self
          .presentational_dependencies
          .push(Box::new(ConstDependency::new(
            ident.span.real_lo(),
            ident.span.real_hi(),
            RuntimeGlobals::SHARE_SCOPE_MAP.name().into(),
            Some(RuntimeGlobals::SHARE_SCOPE_MAP),
          )))
      }
      WEBPACK_INIT_SHARING => {
        self
          .presentational_dependencies
          .push(Box::new(ConstDependency::new(
            ident.span.real_lo(),
            ident.span.real_hi(),
            RuntimeGlobals::INITIALIZE_SHARING.name().into(),
            Some(RuntimeGlobals::INITIALIZE_SHARING),
          )))
      }
      WEBPACK_NONCE => {
        self
          .presentational_dependencies
          .push(Box::new(ConstDependency::new(
            ident.span.real_lo(),
            ident.span.real_hi(),
            RuntimeGlobals::SCRIPT_NONCE.name().into(),
            Some(RuntimeGlobals::SCRIPT_NONCE),
          )));
      }
      WEBPACK_CHUNK_NAME => {
        self
          .presentational_dependencies
          .push(Box::new(ConstDependency::new(
            ident.span.real_lo(),
            ident.span.real_hi(),
            RuntimeGlobals::CHUNK_NAME.name().into(),
            Some(RuntimeGlobals::CHUNK_NAME),
          )));
      }
      RSPACK_VERSION => {
        self
          .presentational_dependencies
          .push(Box::new(ConstDependency::new(
            ident.span.real_lo(),
            ident.span.real_hi(),
            format!("{}()", RuntimeGlobals::RSPACK_VERSION).into(),
            Some(RuntimeGlobals::RSPACK_VERSION),
          )));
      }
      WEBPACK_RUNTIME_ID => {
        self
          .presentational_dependencies
          .push(Box::new(ConstDependency::new(
            ident.span.real_lo(),
            ident.span.real_hi(),
            RuntimeGlobals::RUNTIME_ID.name().into(),
            Some(RuntimeGlobals::RUNTIME_ID),
          )));
      }
      _ => {}
    }
  }

  fn visit_expr(&mut self, expr: &Expr) {
    let span = expr.span();
    if self
      .ignored
      .iter()
      .any(|r| r.start() <= span.real_lo() && span.real_hi() <= r.end())
    {
      return;
    }

    #[macro_export]
    macro_rules! not_supported_expr {
      ($check: ident, $name: literal) => {
        if expr_matcher::$check(expr) {
          let (warning, dep) = super::expression_not_supported(&self.source_file, $name, expr);
          self.warning_diagnostics.push(warning);
          self.presentational_dependencies.push(dep);
          return;
        }
      };
    }

    let root = extract_member_root(expr);

    if let Some(root) = root
      && root.span.ctxt == self.unresolved_ctxt
    {
      if root.sym == "require" {
        not_supported_expr!(is_require_extensions, "require.extensions");
        not_supported_expr!(is_require_ensure, "require.ensure");
        not_supported_expr!(is_require_config, "require.config");
        not_supported_expr!(is_require_version, "require.vesrion");
        not_supported_expr!(is_require_amd, "require.amd");
        not_supported_expr!(is_require_include, "require.include");
        not_supported_expr!(is_require_onerror, "require.onError");
        not_supported_expr!(is_require_main_require, "require.main.require");
      }

      if root.sym == "module" {
        not_supported_expr!(is_module_parent_require, "module.parent.require");
      }
    }

    if expr_matcher::is_require_cache(expr) {
      self
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          expr.span().real_lo(),
          expr.span().real_hi(),
          RuntimeGlobals::MODULE_CACHE.name().into(),
          Some(RuntimeGlobals::MODULE_CACHE),
        )));
    } else if expr_matcher::is_require_main(expr) {
      let mut runtime_requirements = RuntimeGlobals::default();
      runtime_requirements.insert(RuntimeGlobals::MODULE_CACHE);
      runtime_requirements.insert(RuntimeGlobals::ENTRY_MODULE_ID);
      self
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          expr.span().real_lo(),
          expr.span().real_hi(),
          format!(
            "{}[{}]",
            RuntimeGlobals::MODULE_CACHE,
            RuntimeGlobals::ENTRY_MODULE_ID
          )
          .into(),
          Some(runtime_requirements),
        )));
    } else if expr_matcher::is_webpack_module_id(expr) {
      self
        .presentational_dependencies
        .push(Box::new(RuntimeRequirementsDependency::new(
          RuntimeGlobals::MODULE_ID,
        )));
      self
        .presentational_dependencies
        .push(Box::new(ModuleArgumentDependency::new(
          expr.span().real_lo(),
          expr.span().real_hi(),
          Some("id"),
        )));
      return;
    }
    expr.visit_children_with(self);
  }

  fn visit_call_expr(&mut self, call_expr: &CallExpr) {
    #[macro_export]
    macro_rules! not_supported_call {
      ($check: ident, $name: literal) => {
        if let Callee::Expr(box Expr::Member(expr)) = &call_expr.callee
          && expr_matcher::$check(&Expr::Member(expr.to_owned()))
        {
          let (warning, dep) = super::expression_not_supported(
            &self.source_file,
            $name,
            &Expr::Call(call_expr.to_owned()),
          );
          self.warning_diagnostics.push(warning);
          self.presentational_dependencies.push(dep);
          return;
        }
      };
    }

    let root = call_expr
      .callee
      .as_expr()
      .and_then(|expr| extract_member_root(expr));

    if let Some(root) = root
      && root.span.ctxt == self.unresolved_ctxt
    {
      if root.sym == "require" {
        not_supported_call!(is_require_config, "require.config()");
        not_supported_call!(is_require_ensure, "require.ensure()");
        not_supported_call!(is_require_include, "require.include()");
        not_supported_call!(is_require_onerror, "require.onError()");
        not_supported_call!(is_require_main_require, "require.main.require()");
      }

      if root.sym == "module" {
        not_supported_call!(is_module_parent_require, "module.parent.require()");
      }
    }

    call_expr.visit_children_with(self);
  }
}
