use rspack_core::{ConstDependency, RuntimeGlobals, RuntimeRequirementsDependency};
use rspack_error::{Error, Severity};
use rspack_util::SpanExt;
use swc_core::{
  common::{SourceFile, Span, Spanned},
  ecma::ast::{CallExpr, Ident, UnaryExpr},
};

use crate::{
  dependency::ModuleArgumentDependency,
  parser_plugin::JavascriptParserPlugin,
  utils::eval::{self, BasicEvaluatedExpression},
  visitors::{JavascriptParser, create_traceable_error},
};

fn expression_not_supported(
  file: &SourceFile,
  name: &str,
  is_call: bool,
  expr_span: Span,
) -> (Error, Box<ConstDependency>) {
  let mut error = create_traceable_error(
    "Unsupported feature".into(),
    format!(
      "{name}{} is not supported by Rspack.",
      if is_call { "()" } else { "" }
    ),
    file,
    expr_span.into(),
  );
  error.severity = Severity::Warning;
  error.hide_stack = Some(true);
  (
    error,
    Box::new(ConstDependency::new(
      expr_span.into(),
      "(void 0)".into(),
      None,
    )),
  )
}

const WEBPACK_HASH: &str = "__webpack_hash__";
const WEBPACK_LAYER: &str = "__webpack_layer__";
const WEBPACK_PUBLIC_PATH: &str = "__webpack_public_path__";
const WEBPACK_MODULES: &str = "__webpack_modules__";
const WEBPACK_MODULE: &str = "__webpack_module__";
const WEBPACK_CHUNK_LOAD: &str = "__webpack_chunk_load__";
const WEBPACK_BASE_URI: &str = "__webpack_base_uri__";
const NON_WEBPACK_REQUIRE: &str = "__non_webpack_require__";
const SYSTEM_CONTEXT: &str = "__system_context__";
const WEBPACK_SHARE_SCOPES: &str = "__webpack_share_scopes__";
const WEBPACK_INIT_SHARING: &str = "__webpack_init_sharing__";
const WEBPACK_NONCE: &str = "__webpack_nonce__";
const WEBPACK_CHUNK_NAME: &str = "__webpack_chunkname__";
const WEBPACK_RUNTIME_ID: &str = "__webpack_runtime_id__";
const WEBPACK_REQUIRE: &str = RuntimeGlobals::REQUIRE.name();
const WEBPACK_GET_SCRIPT_FILENAME: &str = "__webpack_get_script_filename__";
const RSPACK_VERSION: &str = "__rspack_version__";
const RSPACK_UNIQUE_ID: &str = "__rspack_unique_id__";

pub struct APIPluginOptions {
  module: bool,
}

pub struct APIPlugin {
  options: APIPluginOptions,
}

impl APIPlugin {
  pub fn new(module: bool) -> Self {
    let options = APIPluginOptions { module };
    Self { options }
  }
}

fn get_typeof_evaluate_of_api(sym: &str) -> Option<&str> {
  match sym {
    WEBPACK_REQUIRE => Some("function"),
    WEBPACK_HASH => Some("string"),
    WEBPACK_PUBLIC_PATH => Some("string"),
    WEBPACK_MODULES => Some("object"),
    WEBPACK_MODULE => Some("object"),
    WEBPACK_CHUNK_LOAD => Some("function"),
    WEBPACK_BASE_URI => Some("string"),
    NON_WEBPACK_REQUIRE => None,
    SYSTEM_CONTEXT => Some("object"),
    WEBPACK_SHARE_SCOPES => Some("object"),
    WEBPACK_INIT_SHARING => Some("function"),
    WEBPACK_NONCE => Some("string"),
    WEBPACK_CHUNK_NAME => Some("string"),
    WEBPACK_RUNTIME_ID => None,
    WEBPACK_GET_SCRIPT_FILENAME => Some("function"),
    RSPACK_VERSION => Some("string"),
    RSPACK_UNIQUE_ID => Some("string"),
    _ => None,
  }
}

impl JavascriptParserPlugin for APIPlugin {
  fn evaluate_typeof<'a>(
    &self,
    parser: &mut JavascriptParser,
    expr: &'a UnaryExpr,
    for_name: &str,
  ) -> Option<BasicEvaluatedExpression<'a>> {
    if for_name == WEBPACK_LAYER {
      let value = if parser.module_layer.is_none() {
        "object"
      } else {
        "string"
      };
      Some(eval::evaluate_to_string(
        value.to_string(),
        expr.span.real_lo(),
        expr.span.real_hi(),
      ))
    } else {
      get_typeof_evaluate_of_api(for_name).map(|res| {
        eval::evaluate_to_string(res.to_string(), expr.span.real_lo(), expr.span.real_hi())
      })
    }
  }

  fn identifier(
    &self,
    parser: &mut JavascriptParser,
    ident: &Ident,
    for_name: &str,
  ) -> Option<bool> {
    match for_name {
      WEBPACK_REQUIRE => {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          ident.span.into(),
          RuntimeGlobals::REQUIRE.name().into(),
          Some(RuntimeGlobals::REQUIRE),
        )));
        Some(true)
      }
      WEBPACK_HASH => {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          ident.span.into(),
          format!("{}()", RuntimeGlobals::GET_FULL_HASH).into(),
          Some(RuntimeGlobals::GET_FULL_HASH),
        )));
        Some(true)
      }
      WEBPACK_LAYER => {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          ident.span.into(),
          serde_json::to_string(&parser.module_layer)
            .expect("should stringify JSON")
            .into(),
          None,
        )));
        Some(true)
      }
      WEBPACK_PUBLIC_PATH => {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          ident.span.into(),
          RuntimeGlobals::PUBLIC_PATH.name().into(),
          Some(RuntimeGlobals::PUBLIC_PATH),
        )));
        Some(true)
      }
      WEBPACK_MODULES => {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          ident.span.into(),
          RuntimeGlobals::MODULE_FACTORIES.name().into(),
          Some(RuntimeGlobals::MODULE_FACTORIES),
        )));
        Some(true)
      }
      WEBPACK_CHUNK_LOAD => {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          ident.span.into(),
          RuntimeGlobals::ENSURE_CHUNK.name().into(),
          Some(RuntimeGlobals::ENSURE_CHUNK),
        )));
        Some(true)
      }
      WEBPACK_MODULE => {
        parser.add_presentational_dependency(Box::new(ModuleArgumentDependency::new(
          None,
          ident.span.into(),
          Some(parser.source_map.clone()),
        )));
        Some(true)
      }
      WEBPACK_BASE_URI => {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          ident.span.into(),
          RuntimeGlobals::BASE_URI.name().into(),
          Some(RuntimeGlobals::BASE_URI),
        )));
        Some(true)
      }
      NON_WEBPACK_REQUIRE => {
        let content = if self.options.module {
          parser.build_info.need_create_require = true;
          "__WEBPACK_EXTERNAL_createRequire_require".into()
        } else {
          "require".into()
        };
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          ident.span.into(),
          content,
          None,
        )));
        Some(true)
      }
      SYSTEM_CONTEXT => {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          ident.span.into(),
          RuntimeGlobals::SYSTEM_CONTEXT.name().into(),
          Some(RuntimeGlobals::SYSTEM_CONTEXT),
        )));
        Some(true)
      }
      WEBPACK_SHARE_SCOPES => {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          ident.span.into(),
          RuntimeGlobals::SHARE_SCOPE_MAP.name().into(),
          Some(RuntimeGlobals::SHARE_SCOPE_MAP),
        )));
        Some(true)
      }
      WEBPACK_INIT_SHARING => {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          ident.span.into(),
          RuntimeGlobals::INITIALIZE_SHARING.name().into(),
          Some(RuntimeGlobals::INITIALIZE_SHARING),
        )));
        Some(true)
      }
      WEBPACK_NONCE => {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          ident.span.into(),
          RuntimeGlobals::SCRIPT_NONCE.name().into(),
          Some(RuntimeGlobals::SCRIPT_NONCE),
        )));
        Some(true)
      }
      WEBPACK_CHUNK_NAME => {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          ident.span.into(),
          RuntimeGlobals::CHUNK_NAME.name().into(),
          Some(RuntimeGlobals::CHUNK_NAME),
        )));
        Some(true)
      }
      WEBPACK_RUNTIME_ID => {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          ident.span.into(),
          RuntimeGlobals::RUNTIME_ID.name().into(),
          Some(RuntimeGlobals::RUNTIME_ID),
        )));
        Some(true)
      }
      WEBPACK_GET_SCRIPT_FILENAME => {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          ident.span.into(),
          RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME.name().into(),
          Some(RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME),
        )));
        Some(true)
      }
      // rspack specific
      RSPACK_VERSION => {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          ident.span.into(),
          format!("{}()", RuntimeGlobals::RSPACK_VERSION).into(),
          Some(RuntimeGlobals::RSPACK_VERSION),
        )));
        Some(true)
      }
      RSPACK_UNIQUE_ID => {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          ident.span.into(),
          format!("{}", RuntimeGlobals::RSPACK_UNIQUE_ID).into(),
          Some(RuntimeGlobals::RSPACK_UNIQUE_ID),
        )));
        Some(true)
      }
      _ => None,
    }
  }

  fn evaluate_identifier(
    &self,
    parser: &mut JavascriptParser,
    for_name: &str,
    start: u32,
    end: u32,
  ) -> Option<eval::BasicEvaluatedExpression<'static>> {
    if for_name == WEBPACK_LAYER {
      if let Some(layer) = parser.module_layer {
        Some(eval::evaluate_to_string(layer.into(), start, end))
      } else {
        Some(eval::evaluate_to_null(start, end))
      }
    } else {
      None
    }
  }

  fn member(
    &self,
    parser: &mut JavascriptParser,
    member_expr: &swc_core::ecma::ast::MemberExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == "require.extensions"
      || for_name == "require.config"
      || for_name == "require.version"
      || for_name == "require.include"
      || for_name == "require.onError"
      || for_name == "require.main.require"
      || for_name == "module.parent.require"
    {
      let (warning, dep) =
        expression_not_supported(parser.source_file, for_name, false, member_expr.span());
      parser.add_warning(warning.into());
      parser.add_presentational_dependency(dep);
      return Some(true);
    }

    if for_name == "require.cache" {
      parser.add_presentational_dependency(Box::new(ConstDependency::new(
        member_expr.span().into(),
        RuntimeGlobals::MODULE_CACHE.name().into(),
        Some(RuntimeGlobals::MODULE_CACHE),
      )));
      return Some(true);
    }

    if for_name == "require.main" {
      let mut runtime_requirements = RuntimeGlobals::default();
      runtime_requirements.insert(RuntimeGlobals::MODULE_CACHE);
      runtime_requirements.insert(RuntimeGlobals::ENTRY_MODULE_ID);
      parser.add_presentational_dependency(Box::new(ConstDependency::new(
        member_expr.span().into(),
        format!(
          "{}[{}]",
          RuntimeGlobals::MODULE_CACHE,
          RuntimeGlobals::ENTRY_MODULE_ID
        )
        .into(),
        Some(runtime_requirements),
      )));
      return Some(true);
    }

    if for_name == "__webpack_module__.id" {
      parser.add_presentational_dependency(Box::new(RuntimeRequirementsDependency::new(
        RuntimeGlobals::MODULE_ID,
      )));
      parser.add_presentational_dependency(Box::new(ModuleArgumentDependency::new(
        Some("id".into()),
        member_expr.span().into(),
        Some(parser.source_map.clone()),
      )));
      return Some(true);
    }

    None
  }

  fn call(
    &self,
    parser: &mut JavascriptParser,
    call_expr: &CallExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == "require.config"
      || for_name == "require.include"
      || for_name == "require.onError"
      || for_name == "require.main.require"
      || for_name == "module.parent.require"
    {
      let (warning, dep) =
        expression_not_supported(parser.source_file, for_name, true, call_expr.span());
      parser.add_warning(warning.into());
      parser.add_presentational_dependency(dep);
      return Some(true);
    }

    None
  }
}
