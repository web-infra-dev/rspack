use std::sync::LazyLock;

use regex::Regex;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ApiName {
  Hash,
  Layer,
  PublicPath,
  Modules,
  Module,
  ChunkLoad,
  BaseUri,
  NonRequire,
  SystemContext,
  ShareScopes,
  InitSharing,
  Nonce,
  ChunkName,
  RuntimeId,
  Require,
  GetScriptFilename,
  Version,
  UniqueId,
}

static API_NAME_REGEX: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^__(webpack|rspack)_(.*?)__$").expect("should init regex"));

impl ApiName {
  fn from_str(name: &str, compatibility: bool) -> Option<Self> {
    let Some(caps) = API_NAME_REGEX.captures(name) else {
      if name == "__system_context__" {
        return Some(ApiName::SystemContext);
      }
      if (name == "__non_webpack_require__" && compatibility) || name == "__non_rspack_require__" {
        return Some(ApiName::NonRequire);
      }
      return None;
    };

    let scope = caps.get(1)?.as_str();
    if scope == "webpack" && !compatibility {
      return None;
    }
    let name = caps.get(2)?.as_str();
    match name {
      "hash" => Some(ApiName::Hash),
      "layer" => Some(ApiName::Layer),
      "public_path" => Some(ApiName::PublicPath),
      "modules" => Some(ApiName::Modules),
      "module" => Some(ApiName::Module),
      "chunk_load" => Some(ApiName::ChunkLoad),
      "base_uri" => Some(ApiName::BaseUri),
      "share_scopes" => Some(ApiName::ShareScopes),
      "init_sharing" => Some(ApiName::InitSharing),
      "nonce" => Some(ApiName::Nonce),
      "chunkname" => Some(ApiName::ChunkName),
      "runtime_id" => Some(ApiName::RuntimeId),
      "require" => Some(ApiName::Require),
      "get_script_filename" => Some(ApiName::GetScriptFilename),
      "version" => Some(ApiName::Version),
      "unique_id" => Some(ApiName::UniqueId),
      _ => None,
    }
  }

  fn from_str_with_property(name: &str) -> Option<(Self, String)> {
    let splitted = name.split('.').collect::<Vec<_>>();
    let api_name = Self::from_str(splitted[0], true)?;
    let property = splitted[1..].join(".");
    Some((api_name, property))
  }
}

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

fn get_typeof_evaluate_of_api(api_name: &ApiName) -> Option<&str> {
  match api_name {
    ApiName::Require => Some("function"),
    ApiName::Hash => Some("string"),
    ApiName::PublicPath => Some("string"),
    ApiName::Modules => Some("object"),
    ApiName::Module => Some("object"),
    ApiName::ChunkLoad => Some("function"),
    ApiName::BaseUri => Some("string"),
    ApiName::NonRequire => None,
    ApiName::SystemContext => Some("object"),
    ApiName::ShareScopes => Some("object"),
    ApiName::InitSharing => Some("function"),
    ApiName::Nonce => Some("string"),
    ApiName::ChunkName => Some("string"),
    ApiName::RuntimeId => None,
    ApiName::GetScriptFilename => Some("function"),
    ApiName::Version => Some("string"),
    ApiName::UniqueId => Some("string"),
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
    let api_name = ApiName::from_str(for_name, true)?;
    if api_name == ApiName::Layer {
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
      get_typeof_evaluate_of_api(&api_name).map(|res| {
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
    let api_name = ApiName::from_str(for_name, true)?;
    match api_name {
      ApiName::Require => {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          ident.span.into(),
          RuntimeGlobals::REQUIRE.name().into(),
          Some(RuntimeGlobals::REQUIRE),
        )));
        Some(true)
      }
      ApiName::Hash => {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          ident.span.into(),
          format!("{}()", RuntimeGlobals::GET_FULL_HASH).into(),
          Some(RuntimeGlobals::GET_FULL_HASH),
        )));
        Some(true)
      }
      ApiName::Layer => {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          ident.span.into(),
          serde_json::to_string(&parser.module_layer)
            .expect("should stringify JSON")
            .into(),
          None,
        )));
        Some(true)
      }
      ApiName::PublicPath => {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          ident.span.into(),
          RuntimeGlobals::PUBLIC_PATH.name().into(),
          Some(RuntimeGlobals::PUBLIC_PATH),
        )));
        Some(true)
      }
      ApiName::Modules => {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          ident.span.into(),
          RuntimeGlobals::MODULE_FACTORIES.name().into(),
          Some(RuntimeGlobals::MODULE_FACTORIES),
        )));
        Some(true)
      }
      ApiName::ChunkLoad => {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          ident.span.into(),
          RuntimeGlobals::ENSURE_CHUNK.name().into(),
          Some(RuntimeGlobals::ENSURE_CHUNK),
        )));
        Some(true)
      }
      ApiName::Module => {
        parser.add_presentational_dependency(Box::new(ModuleArgumentDependency::new(
          None,
          ident.span.into(),
          Some(parser.source_map.clone()),
        )));
        Some(true)
      }
      ApiName::BaseUri => {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          ident.span.into(),
          RuntimeGlobals::BASE_URI.name().into(),
          Some(RuntimeGlobals::BASE_URI),
        )));
        Some(true)
      }
      ApiName::NonRequire => {
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
      ApiName::SystemContext => {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          ident.span.into(),
          RuntimeGlobals::SYSTEM_CONTEXT.name().into(),
          Some(RuntimeGlobals::SYSTEM_CONTEXT),
        )));
        Some(true)
      }
      ApiName::ShareScopes => {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          ident.span.into(),
          RuntimeGlobals::SHARE_SCOPE_MAP.name().into(),
          Some(RuntimeGlobals::SHARE_SCOPE_MAP),
        )));
        Some(true)
      }
      ApiName::InitSharing => {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          ident.span.into(),
          RuntimeGlobals::INITIALIZE_SHARING.name().into(),
          Some(RuntimeGlobals::INITIALIZE_SHARING),
        )));
        Some(true)
      }
      ApiName::Nonce => {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          ident.span.into(),
          RuntimeGlobals::SCRIPT_NONCE.name().into(),
          Some(RuntimeGlobals::SCRIPT_NONCE),
        )));
        Some(true)
      }
      ApiName::ChunkName => {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          ident.span.into(),
          RuntimeGlobals::CHUNK_NAME.name().into(),
          Some(RuntimeGlobals::CHUNK_NAME),
        )));
        Some(true)
      }
      ApiName::RuntimeId => {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          ident.span.into(),
          RuntimeGlobals::RUNTIME_ID.name().into(),
          Some(RuntimeGlobals::RUNTIME_ID),
        )));
        Some(true)
      }
      ApiName::GetScriptFilename => {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          ident.span.into(),
          RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME.name().into(),
          Some(RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME),
        )));
        Some(true)
      }
      // rspack specific
      ApiName::Version => {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          ident.span.into(),
          format!("{}()", RuntimeGlobals::RSPACK_VERSION).into(),
          Some(RuntimeGlobals::RSPACK_VERSION),
        )));
        Some(true)
      }
      ApiName::UniqueId => {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          ident.span.into(),
          format!("{}", RuntimeGlobals::RSPACK_UNIQUE_ID).into(),
          Some(RuntimeGlobals::RSPACK_UNIQUE_ID),
        )));
        Some(true)
      }
    }
  }

  fn evaluate_identifier(
    &self,
    parser: &mut JavascriptParser,
    for_name: &str,
    start: u32,
    end: u32,
  ) -> Option<eval::BasicEvaluatedExpression<'static>> {
    let api_name = ApiName::from_str(for_name, true)?;
    if api_name == ApiName::Layer {
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

    if let Some((api_name, property)) = ApiName::from_str_with_property(for_name)
      && api_name == ApiName::Module
      && property == "id"
    {
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
