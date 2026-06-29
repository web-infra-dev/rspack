use rspack_core::{
  ConstDependency, ModuleArgument, RuntimeGlobals, RuntimeRequirementsDependency, property_access,
  runtime_mode::RuntimeMode as ExperimentRuntimeMode,
};
use rspack_error::{Error, Severity};
use rspack_util::{SpanExt, json_stringify_str};
use swc_atoms::Atom;
use swc_experimental_ecma_ast::{
  AssignExpr, AssignOp, CallExpr, GetSpan, Ident, MemberExpr, Pat, Span, UnaryExpr, VarDeclarator,
};

use crate::{
  dependency::{
    ExportInfoDependency, IsIncludeDependency, ModuleArgumentDependency, RequireMainDependency,
  },
  parser_plugin::JavascriptParserPlugin,
  utils::eval::{self, BasicEvaluatedExpression},
  visitors::{JavascriptParser, Statement, VariableDeclaration, create_traceable_error},
};

fn expression_not_supported(
  source: &str,
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
    source.to_owned(),
    expr_span.into(),
  );
  error.severity = Severity::Warning;
  error.hide_stack = Some(true);
  (
    error,
    Box::new(ConstDependency::new(expr_span.into(), "(void 0)".into())),
  )
}

const API_EXPORTS_INFO: &str = "__webpack_exports_info__";
const API_IS_INCLUDED: &str = "__webpack_is_included__";
const API_LAYER: &str = "__webpack_layer__";
const API_MODULE: &str = "__webpack_module__";
const API_NON_REQUIRE: &str = "__non_webpack_require__";
const API_REQUIRE: &str = "__webpack_require__";

#[derive(Clone, Copy)]
enum RuntimeApiIdentifierMode {
  Normal,
  Call,
  Require,
}

#[derive(Clone, Copy)]
struct RuntimeApi {
  name: &'static str,
  type_of: Option<&'static str>,
  runtime_global: Option<RuntimeGlobals>,
  identifier_mode: Option<RuntimeApiIdentifierMode>,
}

static RUNTIME_APIS: &[RuntimeApi] = &[
  RuntimeApi {
    name: API_REQUIRE,
    type_of: Some("function"),
    runtime_global: Some(RuntimeGlobals::REQUIRE),
    identifier_mode: Some(RuntimeApiIdentifierMode::Require),
  },
  RuntimeApi {
    name: "__webpack_hash__",
    type_of: Some("string"),
    runtime_global: Some(RuntimeGlobals::GET_FULL_HASH),
    identifier_mode: Some(RuntimeApiIdentifierMode::Call),
  },
  RuntimeApi {
    name: "__webpack_public_path__",
    type_of: Some("string"),
    runtime_global: Some(RuntimeGlobals::PUBLIC_PATH),
    identifier_mode: Some(RuntimeApiIdentifierMode::Normal),
  },
  RuntimeApi {
    name: "__webpack_modules__",
    type_of: Some("object"),
    runtime_global: Some(RuntimeGlobals::MODULE_FACTORIES),
    identifier_mode: Some(RuntimeApiIdentifierMode::Normal),
  },
  RuntimeApi {
    name: API_MODULE,
    type_of: Some("object"),
    runtime_global: None,
    identifier_mode: None,
  },
  RuntimeApi {
    name: "__webpack_chunk_load__",
    type_of: Some("function"),
    runtime_global: Some(RuntimeGlobals::ENSURE_CHUNK),
    identifier_mode: Some(RuntimeApiIdentifierMode::Normal),
  },
  RuntimeApi {
    name: "__webpack_base_uri__",
    type_of: Some("string"),
    runtime_global: Some(RuntimeGlobals::BASE_URI),
    identifier_mode: Some(RuntimeApiIdentifierMode::Normal),
  },
  RuntimeApi {
    name: API_NON_REQUIRE,
    type_of: None,
    runtime_global: None,
    identifier_mode: None,
  },
  RuntimeApi {
    name: "__system_context__",
    type_of: Some("object"),
    runtime_global: Some(RuntimeGlobals::SYSTEM_CONTEXT),
    identifier_mode: Some(RuntimeApiIdentifierMode::Normal),
  },
  RuntimeApi {
    name: "__webpack_share_scopes__",
    type_of: Some("object"),
    runtime_global: Some(RuntimeGlobals::SHARE_SCOPE_MAP),
    identifier_mode: Some(RuntimeApiIdentifierMode::Normal),
  },
  RuntimeApi {
    name: "__webpack_init_sharing__",
    type_of: Some("function"),
    runtime_global: Some(RuntimeGlobals::INITIALIZE_SHARING),
    identifier_mode: Some(RuntimeApiIdentifierMode::Normal),
  },
  RuntimeApi {
    name: "__webpack_nonce__",
    type_of: Some("string"),
    runtime_global: Some(RuntimeGlobals::SCRIPT_NONCE),
    identifier_mode: Some(RuntimeApiIdentifierMode::Normal),
  },
  RuntimeApi {
    name: "__webpack_chunkname__",
    type_of: Some("string"),
    runtime_global: Some(RuntimeGlobals::CHUNK_NAME),
    identifier_mode: Some(RuntimeApiIdentifierMode::Normal),
  },
  RuntimeApi {
    name: "__webpack_runtime_id__",
    type_of: None,
    runtime_global: Some(RuntimeGlobals::RUNTIME_ID),
    identifier_mode: Some(RuntimeApiIdentifierMode::Normal),
  },
  RuntimeApi {
    name: "__webpack_get_script_filename__",
    type_of: Some("function"),
    runtime_global: Some(RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME),
    identifier_mode: Some(RuntimeApiIdentifierMode::Normal),
  },
  RuntimeApi {
    name: "__rspack_version__",
    type_of: Some("string"),
    runtime_global: Some(RuntimeGlobals::RSPACK_VERSION),
    identifier_mode: Some(RuntimeApiIdentifierMode::Call),
  },
  RuntimeApi {
    name: "__rspack_unique_id__",
    type_of: Some("string"),
    runtime_global: Some(RuntimeGlobals::RSPACK_UNIQUE_ID),
    identifier_mode: Some(RuntimeApiIdentifierMode::Normal),
  },
  RuntimeApi {
    name: "__rspack_rsc_manifest__",
    type_of: Some("object"),
    runtime_global: Some(RuntimeGlobals::RSC_MANIFEST),
    identifier_mode: Some(RuntimeApiIdentifierMode::Normal),
  },
];

#[derive(Clone, Copy)]
pub(crate) struct ImportMetaRuntimeApi {
  pub(crate) name: &'static str,
  pub(crate) property: &'static str,
  pub(crate) type_of: &'static str,
  runtime_global: RuntimeGlobals,
  runtime_call: bool,
}

static IMPORT_META_RUNTIME_APIS: &[ImportMetaRuntimeApi] = &[
  ImportMetaRuntimeApi {
    name: "import.meta.rspackPublicPath",
    property: "rspackPublicPath",
    type_of: "string",
    runtime_global: RuntimeGlobals::PUBLIC_PATH,
    runtime_call: false,
  },
  ImportMetaRuntimeApi {
    name: "import.meta.rspackBaseUri",
    property: "rspackBaseUri",
    type_of: "string",
    runtime_global: RuntimeGlobals::BASE_URI,
    runtime_call: false,
  },
  ImportMetaRuntimeApi {
    name: "import.meta.rspackShareScopes",
    property: "rspackShareScopes",
    type_of: "object",
    runtime_global: RuntimeGlobals::SHARE_SCOPE_MAP,
    runtime_call: false,
  },
  ImportMetaRuntimeApi {
    name: "import.meta.rspackInitSharing",
    property: "rspackInitSharing",
    type_of: "function",
    runtime_global: RuntimeGlobals::INITIALIZE_SHARING,
    runtime_call: false,
  },
  ImportMetaRuntimeApi {
    name: "import.meta.rspackNonce",
    property: "rspackNonce",
    type_of: "string",
    runtime_global: RuntimeGlobals::SCRIPT_NONCE,
    runtime_call: false,
  },
  ImportMetaRuntimeApi {
    name: "import.meta.rspackUniqueId",
    property: "rspackUniqueId",
    type_of: "string",
    runtime_global: RuntimeGlobals::RSPACK_UNIQUE_ID,
    runtime_call: false,
  },
  ImportMetaRuntimeApi {
    name: "import.meta.rspackVersion",
    property: "rspackVersion",
    type_of: "string",
    runtime_global: RuntimeGlobals::RSPACK_VERSION,
    runtime_call: true,
  },
  ImportMetaRuntimeApi {
    name: "import.meta.rspackHash",
    property: "rspackHash",
    type_of: "string",
    runtime_global: RuntimeGlobals::GET_FULL_HASH,
    runtime_call: true,
  },
];

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

fn runtime_api_from_name(name: &str) -> Option<&'static RuntimeApi> {
  RUNTIME_APIS.iter().find(|api| api.name == name)
}

fn get_typeof_evaluate_of_api(sym: &str) -> Option<&'static str> {
  runtime_api_from_name(sym).and_then(|api| api.type_of)
}

pub(crate) fn import_meta_runtime_api_from_name(
  name: &str,
) -> Option<&'static ImportMetaRuntimeApi> {
  IMPORT_META_RUNTIME_APIS.iter().find(|api| api.name == name)
}

pub(crate) fn import_meta_runtime_api_from_property(
  property: &str,
) -> Option<&'static ImportMetaRuntimeApi> {
  IMPORT_META_RUNTIME_APIS
    .iter()
    .find(|api| api.property == property)
}

pub(crate) fn render_import_meta_runtime_api(
  parser: &JavascriptParser,
  api: &ImportMetaRuntimeApi,
) -> Option<String> {
  let content = if parser.compiler_options.experiments.runtime_mode == ExperimentRuntimeMode::Rspack
  {
    format!(
      "{}{}",
      parser.parser_runtime_requirements.context,
      property_access([api.runtime_global.rspack_context_property_name()?], 0)
    )
  } else {
    format!(
      "{}{}",
      parser.parser_runtime_requirements.require,
      property_access([api.runtime_global.property_name()?], 0)
    )
  };
  Some(if api.runtime_call {
    format!("{content}()")
  } else {
    content
  })
}

pub(crate) fn render_import_meta_runtime_api_destructuring(
  parser: &mut JavascriptParser,
  api: &ImportMetaRuntimeApi,
) -> Option<String> {
  parser.add_presentational_dependency(Box::new(RuntimeRequirementsDependency::add_only(
    api.runtime_global,
  )));
  Some(format!(
    "{}: {}",
    api.property,
    render_import_meta_runtime_api(parser, api)?
  ))
}

pub(crate) fn import_meta_runtime_api_member(
  parser: &mut JavascriptParser,
  span: Span,
  api: &ImportMetaRuntimeApi,
) -> Option<bool> {
  let dep = if api.runtime_call {
    RuntimeRequirementsDependency::call(span.into(), api.runtime_global)
  } else {
    RuntimeRequirementsDependency::new(span.into(), api.runtime_global)
  };
  parser.add_presentational_dependency(Box::new(dep));
  Some(true)
}

pub(crate) fn import_meta_runtime_api_call(
  parser: &mut JavascriptParser,
  call_expr: &CallExpr,
  api: &ImportMetaRuntimeApi,
) -> Option<bool> {
  if api.type_of != "function" {
    return None;
  }
  parser.add_presentational_dependency(Box::new(RuntimeRequirementsDependency::new(
    call_expr.callee.span().into(),
    api.runtime_global,
  )));
  parser.walk_expr_or_spread(&call_expr.args);
  Some(true)
}

pub(crate) fn import_meta_runtime_api_assign(
  parser: &mut JavascriptParser,
  span: Span,
  api: &ImportMetaRuntimeApi,
  full_assignment: bool,
  simple_assignment: bool,
) -> Option<bool> {
  if api.runtime_call {
    let content = if full_assignment {
      if simple_assignment {
        format!("({{}}).{}", api.property)
      } else {
        parser.add_presentational_dependency(Box::new(RuntimeRequirementsDependency::add_only(
          api.runtime_global,
        )));
        format!(
          "({{ {}: {} }}).{}",
          api.property,
          render_import_meta_runtime_api(parser, api)?,
          api.property
        )
      }
    } else {
      "({})".to_string()
    };
    parser
      .add_presentational_dependency(Box::new(ConstDependency::new(span.into(), content.into())));
    return Some(true);
  }
  parser.add_presentational_dependency(Box::new(RuntimeRequirementsDependency::write(
    span.into(),
    api.runtime_global,
  )));
  Some(true)
}

pub(crate) fn is_simple_assign_op(op: AssignOp) -> bool {
  matches!(op, AssignOp::Assign)
}

fn static_require_member_chain(
  parser: &mut JavascriptParser,
  for_name: &str,
  members: &[Atom],
  member_ranges: Option<&[Span]>,
  expr_span: Span,
  write: bool,
) -> Option<bool> {
  if parser.compiler_options.experiments.runtime_mode != ExperimentRuntimeMode::Rspack {
    return None;
  }

  if for_name == API_REQUIRE
    && let Some(property) = members.first()
  {
    if let Some(runtime_global) =
      RuntimeGlobals::from_rspack_context_property_name(property.as_ref())
    {
      let dep_span = if members.len() > 1 {
        member_ranges
          .and_then(|ranges| ranges.get(1))
          .map_or(expr_span, |range| Span::new(expr_span.start, range.end))
      } else {
        expr_span
      };
      let dep = if write {
        RuntimeRequirementsDependency::write(dep_span.into(), runtime_global)
      } else {
        RuntimeRequirementsDependency::new(dep_span.into(), runtime_global)
      };
      parser.add_presentational_dependency(Box::new(dep));
    } else {
      let content = format!(
        "{}{}",
        parser.parser_runtime_requirements.context,
        property_access(members.iter().map(Atom::as_ref), 0)
      );
      parser.add_presentational_dependency(Box::new(RuntimeRequirementsDependency::add_only(
        RuntimeGlobals::REQUIRE_SCOPE,
      )));
      parser.add_presentational_dependency(Box::new(ConstDependency::new(
        expr_span.into(),
        content.into(),
      )));
    }
    return Some(true);
  }

  None
}

#[rspack_macros::implemented_javascript_parser_hooks]
impl<'p, 'a> JavascriptParserPlugin<'p, 'a> for APIPlugin {
  fn r#typeof(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: &UnaryExpr,
    for_name: &str,
  ) -> Option<bool> {
    (for_name == API_IS_INCLUDED).then(|| {
      parser.add_presentational_dependency(Box::new(ConstDependency::new(
        (expr.span.real_lo(), expr.span.real_hi()).into(),
        "'function'".into(),
      )));
      true
    })
  }

  fn evaluate_typeof(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: &'a UnaryExpr<'a>,
    for_name: &str,
  ) -> Option<BasicEvaluatedExpression<'a>> {
    if for_name == API_LAYER {
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
    parser: &mut JavascriptParser<'p>,
    ident: &Ident,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == API_LAYER {
      parser.add_presentational_dependency(Box::new(ConstDependency::new(
        ident.span.into(),
        serde_json::to_string(&parser.module_layer)
          .expect("should stringify JSON")
          .into(),
      )));
      return Some(true);
    }

    if for_name == API_MODULE {
      let range = ident.span.into();
      let loc = parser.to_dependency_location(range);
      parser
        .add_presentational_dependency(Box::new(ModuleArgumentDependency::new(None, range, loc)));
      return Some(true);
    }

    if for_name == API_NON_REQUIRE {
      let content = if self.options.module {
        parser.build_info.need_create_require = true;
        "__rspack_createRequire_require".into()
      } else {
        "require".into()
      };
      parser
        .add_presentational_dependency(Box::new(ConstDependency::new(ident.span.into(), content)));
      return Some(true);
    }

    if for_name == API_EXPORTS_INFO {
      let dep = Box::new(ConstDependency::new(ident.span.into(), "true".into()));
      parser.add_presentational_dependency(dep);
      return Some(true);
    }

    let api = runtime_api_from_name(for_name)?;
    let runtime_global = api
      .runtime_global
      .expect("runtime api identifier should have runtime global");
    match for_name {
      _ if matches!(api.identifier_mode, Some(RuntimeApiIdentifierMode::Require))
        && parser.compiler_options.experiments.runtime_mode == ExperimentRuntimeMode::Rspack =>
      {
        parser.add_presentational_dependency(Box::new(RuntimeRequirementsDependency::new(
          ident.span.into(),
          runtime_global,
        )));
        Some(true)
      }
      _ if matches!(api.identifier_mode, Some(RuntimeApiIdentifierMode::Require)) => {
        parser.add_presentational_dependency(Box::new(RuntimeRequirementsDependency::add_only(
          runtime_global,
        )));
        None
      }
      _ if matches!(api.identifier_mode, Some(RuntimeApiIdentifierMode::Call)) => {
        parser.add_presentational_dependency(Box::new(RuntimeRequirementsDependency::call(
          ident.span.into(),
          runtime_global,
        )));
        Some(true)
      }
      API_LAYER => {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          ident.span.into(),
          parser
            .module_layer
            .map_or_else(|| "null".to_string(), |layer| json_stringify_str(layer))
            .into(),
        )));
        Some(true)
      }
      _ if matches!(api.identifier_mode, Some(RuntimeApiIdentifierMode::Normal)) => {
        parser.add_presentational_dependency(Box::new(RuntimeRequirementsDependency::new(
          ident.span.into(),
          runtime_global,
        )));
        Some(true)
      }
      _ => None,
    }
  }

  fn evaluate_identifier(
    &self,
    parser: &mut JavascriptParser<'p>,
    for_name: &str,
    _member_expr_info: Option<&crate::visitors::ExpressionExpressionInfo>,
    start: u32,
    end: u32,
  ) -> Option<eval::BasicEvaluatedExpression<'p>> {
    if for_name == API_LAYER {
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
    parser: &mut JavascriptParser<'p>,
    member_expr: &MemberExpr,
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
        expression_not_supported(parser.source, for_name, false, member_expr.span());
      parser.add_warning(warning.into());
      parser.add_presentational_dependency(dep);
      return Some(true);
    }

    if for_name == "require.cache" {
      parser.add_presentational_dependency(Box::new(RuntimeRequirementsDependency::new(
        member_expr.span().into(),
        RuntimeGlobals::MODULE_CACHE,
      )));
      return Some(true);
    }

    if for_name == "require.main" {
      parser.add_presentational_dependency(Box::new(RequireMainDependency::new(
        member_expr.span().into(),
      )));
      return Some(true);
    }

    if for_name == "__webpack_module__.id" {
      let range = member_expr.span.into();
      let loc = parser.to_dependency_location(range);
      parser.add_presentational_dependency(Box::new(ModuleArgumentDependency::new(
        Some("id".into()),
        range,
        loc,
      )));
      return Some(true);
    }

    None
  }

  fn member_chain(
    &self,
    parser: &mut JavascriptParser,
    member_expr: &MemberExpr,
    for_name: &str,
    members: &[Atom],
    _members_optionals: &[bool],
    member_ranges: &[Span],
  ) -> Option<bool> {
    let len = members.len();
    if len >= 1 && for_name == API_EXPORTS_INFO {
      let prop = members[len - 1].clone();
      let dep = Box::new(ExportInfoDependency::new(
        member_expr.span.real_lo(),
        member_expr.span.real_hi(),
        members.iter().take(len - 1).cloned().collect::<Vec<_>>(),
        prop,
      ));
      parser.add_presentational_dependency(dep);
      return Some(true);
    }

    if parser.compiler_options.experiments.runtime_mode != ExperimentRuntimeMode::Rspack {
      return None;
    }
    static_require_member_chain(
      parser,
      for_name,
      members,
      Some(member_ranges),
      member_expr.span,
      false,
    )
  }

  fn call_member_chain(
    &self,
    parser: &mut JavascriptParser,
    expr: &CallExpr,
    for_name: &str,
    members: &[Atom],
    _members_optionals: &[bool],
    member_ranges: &[Span],
  ) -> Option<bool> {
    if parser.compiler_options.experiments.runtime_mode != ExperimentRuntimeMode::Rspack {
      return None;
    }
    let handled = static_require_member_chain(
      parser,
      for_name,
      members,
      Some(member_ranges),
      expr.callee.span(),
      false,
    );
    if handled.is_some() {
      parser.walk_expr_or_spread(&expr.args);
    }
    handled
  }

  fn assign_member_chain(
    &self,
    parser: &mut JavascriptParser,
    expr: &AssignExpr,
    members: &[Atom],
    member_ranges: &[Span],
    for_name: &str,
  ) -> Option<bool> {
    if parser.compiler_options.experiments.runtime_mode != ExperimentRuntimeMode::Rspack {
      return None;
    }
    let handled = static_require_member_chain(
      parser,
      for_name,
      members,
      Some(member_ranges),
      expr.left.span(),
      true,
    );
    if handled.is_some() {
      parser.walk_expression(&expr.right);
    }
    handled
  }

  fn assign(
    &self,
    parser: &mut JavascriptParser,
    _expr: &AssignExpr,
    _ident: &Ident,
    for_name: &str,
  ) -> Option<bool> {
    if let Some(runtime_global) = runtime_api_from_name(for_name).and_then(|api| api.runtime_global)
      && parser.compiler_options.experiments.runtime_mode == ExperimentRuntimeMode::Rspack
    {
      parser.add_presentational_dependency(Box::new(RuntimeRequirementsDependency::write_only(
        runtime_global,
      )));
    }
    None
  }

  fn pre_declarator(
    &self,
    parser: &mut JavascriptParser<'p>,
    declarator: &VarDeclarator,
    _declaration: VariableDeclaration<'_>,
  ) -> Option<bool> {
    // Check if we're at top level scope and the declarator is a simple identifier named "module"
    if parser.is_top_level_scope()
      && let Pat::Ident(ident) = &declarator.name
      && ident.id.sym.as_ref() == "module"
    {
      parser.build_info.module_argument = ModuleArgument::RspackModule;
    }
    None
  }

  fn pre_statement(&self, parser: &mut JavascriptParser<'p>, stmt: Statement) -> Option<bool> {
    // Check if we're at top level scope
    if parser.is_top_level_scope() {
      match stmt {
        Statement::Fn(fn_decl) => {
          // Check for function declaration named "module"
          if let Some(ident) = fn_decl.ident()
            && ident.sym.as_ref() == "module"
          {
            parser.build_info.module_argument = ModuleArgument::RspackModule;
          }
        }
        Statement::Class(class_decl) => {
          // Check for class declaration named "module"
          if let Some(ident) = class_decl.ident()
            && ident.sym.as_ref() == "module"
          {
            parser.build_info.module_argument = ModuleArgument::RspackModule;
          }
        }
        _ => {}
      }
    }
    None
  }

  fn call(
    &self,
    parser: &mut JavascriptParser<'p>,
    call_expr: &CallExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == API_IS_INCLUDED
      && call_expr.args.len() == 1
      && call_expr.args[0].spread.is_none()
    {
      let request = parser.evaluate_expression(&call_expr.args[0].expr);
      if request.is_string() {
        parser.add_dependency(Box::new(IsIncludeDependency::new(
          (call_expr.span.real_lo(), call_expr.span.real_hi()).into(),
          request.string().clone(),
        )));
        return Some(true);
      }
    }

    if for_name == API_REQUIRE
      && parser.compiler_options.experiments.runtime_mode == ExperimentRuntimeMode::Rspack
    {
      parser.add_presentational_dependency(Box::new(RuntimeRequirementsDependency::add_only(
        RuntimeGlobals::REQUIRE,
      )));
      return None;
    }

    if for_name == "require.config"
      || for_name == "require.include"
      || for_name == "require.onError"
      || for_name == "require.main.require"
      || for_name == "module.parent.require"
    {
      let (warning, dep) = expression_not_supported(parser.source, for_name, true, call_expr.span);
      parser.add_warning(warning.into());
      parser.add_presentational_dependency(dep);
      return Some(true);
    }

    None
  }
}
