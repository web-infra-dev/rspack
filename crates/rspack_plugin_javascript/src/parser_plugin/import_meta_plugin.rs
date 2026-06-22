use itertools::Itertools;
use rspack_core::{
  ConstDependency, ContextDependency, ContextMode, ContextOptions, DependencyCategory,
  DependencyRange, ImportMeta, RscMeta, RscModuleType, RuntimeGlobals,
  RuntimeRequirementsDependency, property_access,
};
use rspack_error::{Error, Severity};
use rspack_util::SpanExt;
use swc_experimental_ecma_ast::{
  CallExpr, Expr, GetSpan, MemberExpr, MemberProp, MetaPropKind, Span, UnaryExpr,
};
use url::Url;

use super::JavascriptParserPlugin;
use crate::{
  dependency::{
    IMPORT_META_RSC_BINDING, ImportMetaResolveContextDependency, ImportMetaResolveDependency,
    ImportMetaResolveHeaderDependency, ImportMetaRscDependency,
  },
  utils::eval::{self, BasicEvaluatedExpression},
  visitors::{
    AllowedMemberTypes, ExportedVariableInfo, ExprRef, JavascriptParser, MemberExpressionInfo,
    RootName, context_reg_exp, create_context_dependency, create_traceable_error, expr_name,
  },
};

fn create_import_meta_resolve_context_dependency(
  parser: &mut JavascriptParser,
  param: &BasicEvaluatedExpression,
  range: DependencyRange,
) -> ImportMetaResolveContextDependency {
  let start = range.start;
  let end = range.end;
  let result = create_context_dependency(param, parser);

  let options = ContextOptions {
    mode: ContextMode::Sync,
    recursive: true,
    pattern: context_reg_exp(&result.reg, "", None, parser).into(),
    category: DependencyCategory::Esm,
    request: format!("{}{}{}", result.context, result.query, result.fragment),
    context: result.context,
    replaces: result.replaces,
    start,
    end,
    ..Default::default()
  };
  let mut dep = ImportMetaResolveContextDependency::new(options, range, parser.in_try);
  *dep.critical_mut() = result.critical;
  dep
}

type ImportMetaApiCondition = for<'p> fn(&JavascriptParser<'p>) -> bool;
type ImportMetaEvaluateIdentifier = for<'p> fn(
  &ImportMetaPlugin,
  &mut JavascriptParser<'p>,
  u32,
  u32,
) -> BasicEvaluatedExpression<'static>;
type ImportMetaMember =
  for<'p> fn(&ImportMetaPlugin, &mut JavascriptParser<'p>, &MemberExpr) -> bool;
type ImportMetaDestructuring =
  for<'p> fn(&ImportMetaPlugin, &mut JavascriptParser<'p>, &'static str, Span) -> String;
type ImportMetaCall = for<'p> fn(&ImportMetaPlugin, &mut JavascriptParser<'p>, &CallExpr) -> bool;
type ImportMetaRuntimeMember = for<'p> fn(&mut JavascriptParser<'p>, Span, RuntimeGlobals) -> bool;

#[derive(Clone, Copy)]
struct ImportMetaApi {
  name: &'static str,
  property: &'static str,
  type_of: Option<&'static str>,
  replace_typeof: bool,
  evaluate_identifier: Option<ImportMetaEvaluateIdentifier>,
  member: Option<ImportMetaMember>,
  destructuring: Option<ImportMetaDestructuring>,
  call: Option<ImportMetaCall>,
  skip_undefined_evaluate: bool,
  condition: ImportMetaApiCondition,
  runtime_global: Option<RuntimeGlobals>,
  runtime_member: Option<ImportMetaRuntimeMember>,
}

static IMPORT_META_APIS: &[ImportMetaApi] = &[
  ImportMetaApi {
    name: expr_name::IMPORT_META_URL,
    property: "url",
    type_of: Some("string"),
    replace_typeof: true,
    evaluate_identifier: Some(|plugin, parser, start, end| {
      eval::evaluate_to_string(plugin.import_meta_url(parser), start, end)
    }),
    member: Some(|plugin, parser, member_expr| {
      parser.add_presentational_dependency(Box::new(ConstDependency::new(
        member_expr.span().into(),
        format!("'{}'", plugin.import_meta_url(parser)).into(),
      )));
      true
    }),
    destructuring: Some(|plugin, parser, property, _span| {
      format!(r#"{property}: "{}""#, plugin.import_meta_url(parser))
    }),
    call: None,
    skip_undefined_evaluate: false,
    condition: |_| true,
    runtime_global: None,
    runtime_member: None,
  },
  ImportMetaApi {
    name: expr_name::IMPORT_META_RESOLVE,
    property: "resolve",
    type_of: Some("function"),
    replace_typeof: true,
    evaluate_identifier: Some(|_plugin, _parser, start, end| {
      eval::evaluate_to_identifier(
        expr_name::IMPORT_META_RESOLVE.into(),
        expr_name::IMPORT_META_RESOLVE.into(),
        Some(true),
        start,
        end,
      )
    }),
    member: None,
    destructuring: None,
    call: Some(|plugin, parser, call_expr| {
      plugin.process_import_meta_resolve(parser, call_expr);
      true
    }),
    skip_undefined_evaluate: false,
    condition: |parser| parser.javascript_options.import_meta_resolve == Some(true),
    runtime_global: None,
    runtime_member: None,
  },
  ImportMetaApi {
    name: expr_name::IMPORT_META_VERSION,
    property: "webpack",
    type_of: Some("number"),
    replace_typeof: true,
    evaluate_identifier: Some(|_plugin, _parser, start, end| {
      eval::evaluate_to_number(5_f64, start, end)
    }),
    member: Some(|plugin, parser, member_expr| {
      parser.add_presentational_dependency(Box::new(ConstDependency::new(
        member_expr.span().into(),
        plugin.import_meta_version().into(),
      )));
      true
    }),
    destructuring: Some(|plugin, _parser, property, _span| {
      format!("{property}: {}", plugin.import_meta_version())
    }),
    call: None,
    skip_undefined_evaluate: false,
    condition: |_| true,
    runtime_global: None,
    runtime_member: None,
  },
  ImportMetaApi {
    name: expr_name::IMPORT_META_MAIN,
    property: "main",
    type_of: Some("boolean"),
    replace_typeof: true,
    evaluate_identifier: None,
    member: Some(|plugin, parser, member_expr| {
      let content = plugin.import_meta_main(parser);
      parser.add_presentational_dependency(Box::new(ConstDependency::new(
        member_expr.span().into(),
        content.into(),
      )));
      true
    }),
    destructuring: Some(|plugin, parser, property, _span| {
      format!("{property}: {}", plugin.import_meta_main(parser))
    }),
    call: None,
    skip_undefined_evaluate: true,
    condition: |_| true,
    runtime_global: None,
    runtime_member: None,
  },
  ImportMetaApi {
    name: expr_name::IMPORT_META_RSPACK_RSC,
    property: "rspackRsc",
    type_of: Some("object"),
    replace_typeof: false,
    evaluate_identifier: None,
    member: Some(|plugin, parser, member_expr| {
      plugin.process_rspack_rsc(parser, member_expr);
      true
    }),
    destructuring: Some(|plugin, parser, property, span| {
      format!(
        "{property}: {}",
        plugin.process_rspack_rsc_destructuring(parser, span)
      )
    }),
    call: None,
    skip_undefined_evaluate: true,
    condition: is_rsc_layer,
    runtime_global: None,
    runtime_member: None,
  },
  ImportMetaApi {
    name: "import.meta.rspackPublicPath",
    property: "rspackPublicPath",
    type_of: Some("string"),
    replace_typeof: true,
    evaluate_identifier: None,
    member: None,
    destructuring: None,
    call: None,
    skip_undefined_evaluate: true,
    condition: |_| true,
    runtime_global: Some(RuntimeGlobals::PUBLIC_PATH),
    runtime_member: Some(normal_import_meta_runtime_member),
  },
  ImportMetaApi {
    name: "import.meta.rspackBaseUri",
    property: "rspackBaseUri",
    type_of: Some("string"),
    replace_typeof: true,
    evaluate_identifier: None,
    member: None,
    destructuring: None,
    call: None,
    skip_undefined_evaluate: true,
    condition: |_| true,
    runtime_global: Some(RuntimeGlobals::BASE_URI),
    runtime_member: Some(normal_import_meta_runtime_member),
  },
  ImportMetaApi {
    name: "import.meta.rspackShareScopes",
    property: "rspackShareScopes",
    type_of: Some("object"),
    replace_typeof: true,
    evaluate_identifier: None,
    member: None,
    destructuring: None,
    call: None,
    skip_undefined_evaluate: true,
    condition: |_| true,
    runtime_global: Some(RuntimeGlobals::SHARE_SCOPE_MAP),
    runtime_member: Some(normal_import_meta_runtime_member),
  },
  ImportMetaApi {
    name: "import.meta.rspackInitSharing",
    property: "rspackInitSharing",
    type_of: Some("function"),
    replace_typeof: true,
    evaluate_identifier: None,
    member: None,
    destructuring: None,
    call: None,
    skip_undefined_evaluate: true,
    condition: |_| true,
    runtime_global: Some(RuntimeGlobals::INITIALIZE_SHARING),
    runtime_member: Some(normal_import_meta_runtime_member),
  },
  ImportMetaApi {
    name: "import.meta.rspackNonce",
    property: "rspackNonce",
    type_of: Some("string"),
    replace_typeof: true,
    evaluate_identifier: None,
    member: None,
    destructuring: None,
    call: None,
    skip_undefined_evaluate: true,
    condition: |_| true,
    runtime_global: Some(RuntimeGlobals::SCRIPT_NONCE),
    runtime_member: Some(normal_import_meta_runtime_member),
  },
  ImportMetaApi {
    name: "import.meta.rspackVersion",
    property: "rspackVersion",
    type_of: Some("string"),
    replace_typeof: true,
    evaluate_identifier: None,
    member: None,
    destructuring: None,
    call: None,
    skip_undefined_evaluate: true,
    condition: |_| true,
    runtime_global: Some(RuntimeGlobals::RSPACK_VERSION),
    runtime_member: Some(call_import_meta_runtime_member),
  },
  ImportMetaApi {
    name: "import.meta.rspackHash",
    property: "rspackHash",
    type_of: Some("string"),
    replace_typeof: true,
    evaluate_identifier: None,
    member: None,
    destructuring: None,
    call: None,
    skip_undefined_evaluate: true,
    condition: |_| true,
    runtime_global: Some(RuntimeGlobals::GET_FULL_HASH),
    runtime_member: Some(call_import_meta_runtime_member),
  },
];

impl ImportMetaApi {
  fn member(
    &self,
    plugin: &ImportMetaPlugin,
    parser: &mut JavascriptParser,
    member_expr: &MemberExpr,
  ) -> Option<bool> {
    if let Some(member) = self.member {
      return Some(member(plugin, parser, member_expr));
    }
    let runtime_member = self.runtime_member?;
    Some(runtime_member(
      parser,
      member_expr.span(),
      self.runtime_global?,
    ))
  }
}

fn normal_import_meta_runtime_member(
  parser: &mut JavascriptParser,
  span: Span,
  runtime_global: RuntimeGlobals,
) -> bool {
  parser.add_presentational_dependency(Box::new(RuntimeRequirementsDependency::new(
    span.into(),
    runtime_global,
  )));
  true
}

fn call_import_meta_runtime_member(
  parser: &mut JavascriptParser,
  span: Span,
  runtime_global: RuntimeGlobals,
) -> bool {
  parser.add_presentational_dependency(Box::new(RuntimeRequirementsDependency::call(
    span.into(),
    runtime_global,
  )));
  true
}

fn import_meta_api_from_name(name: &str) -> Option<&'static ImportMetaApi> {
  IMPORT_META_APIS.iter().find(|api| api.name == name)
}

fn import_meta_api_from_property(property: &str) -> Option<&'static ImportMetaApi> {
  IMPORT_META_APIS.iter().find(|api| api.property == property)
}

fn should_skip_import_meta_undefined_evaluate(parser: &JavascriptParser, property: &str) -> bool {
  // `dirname` and `filename` are handled by NodeStuffPlugin and may have runtime
  // values when node.__dirname/node.__filename is false.
  property == "dirname"
    || property == "filename"
    || import_meta_api_from_property(property)
      .is_some_and(|api| api.skip_undefined_evaluate && (api.condition)(parser))
}

pub struct ImportMetaPlugin(pub(crate) ImportMeta);

impl ImportMetaPlugin {
  fn import_meta_url(&self, parser: &JavascriptParser) -> String {
    Url::from_file_path(parser.resource_data.resource())
      .expect("should be a path")
      .to_string()
  }

  fn import_meta_version(&self) -> String {
    "5".to_string()
  }

  fn import_meta_main(&self, parser: &mut JavascriptParser) -> String {
    parser.build_info.module_concatenation_bailout = Some("import.meta.main".into());
    parser.add_presentational_dependency(Box::new(RuntimeRequirementsDependency::add_only(
      RuntimeGlobals::MODULE_CACHE | RuntimeGlobals::ENTRY_MODULE_ID | RuntimeGlobals::MODULE,
    )));
    format!(
      "({}[{}] === {})",
      parser.parser_runtime_requirements.module_cache,
      parser.parser_runtime_requirements.entry_module_id,
      parser
        .parser_runtime_requirements
        .module_argument(&parser.build_info.module_argument)
    )
  }

  fn import_meta_unknown_property(&self, members: &Vec<String>) -> String {
    if matches!(self.0, ImportMeta::PreserveUnknown) {
      format!("import.meta{}", property_access(members, 0))
    } else {
      format!(
        r#"/* unsupported import.meta.{} */ undefined{}"#,
        members.join("."),
        property_access(members, 1)
      )
    }
  }

  fn process_import_meta_resolve(&self, parser: &mut JavascriptParser, call_expr: &CallExpr) {
    if call_expr.args.len() != 1 {
      return;
    }

    let argument_expr = &call_expr.args[0].expr;
    let param = parser.evaluate_expression(argument_expr);
    let callee_span = call_expr.callee.span();
    let range = DependencyRange::from(callee_span);
    let loc = parser.to_dependency_location(range);
    let import_meta_resolve_header_dependency = Box::new(ImportMetaResolveHeaderDependency::new(
      callee_span.into(),
      loc,
    ));

    if param.is_conditional() {
      for option in param.options() {
        if !self.process_import_meta_resolve_item(parser, option) {
          self.process_import_meta_resolve_context(parser, option);
        }
      }
    } else if !self.process_import_meta_resolve_item(parser, &param) {
      self.process_import_meta_resolve_context(parser, &param);
    }
    parser.add_dependency(import_meta_resolve_header_dependency);
  }

  fn process_import_meta_resolve_item(
    &self,
    parser: &mut JavascriptParser,
    param: &eval::BasicEvaluatedExpression,
  ) -> bool {
    if param.is_string() {
      parser.add_dependency(Box::new(ImportMetaResolveDependency::new(
        param.string().clone(),
        param.range().into(),
        parser.in_try,
      )));
      return true;
    }

    false
  }

  fn process_import_meta_resolve_context(
    &self,
    parser: &mut JavascriptParser,
    param: &BasicEvaluatedExpression,
  ) {
    let dep = create_import_meta_resolve_context_dependency(parser, param, param.range().into());
    parser.add_dependency(Box::new(dep));
  }

  fn process_rspack_rsc(&self, parser: &mut JavascriptParser, member_expr: &MemberExpr) {
    let importer = get_rspack_rsc_importer(parser);
    mark_import_meta_rsc_used(parser);

    let range = member_expr.span().into();
    let loc = parser.to_dependency_location(range);
    parser.add_dependency(Box::new(ImportMetaRscDependency::new(importer, range, loc)));
  }

  fn process_rspack_rsc_destructuring(&self, parser: &mut JavascriptParser, span: Span) -> String {
    let importer = get_rspack_rsc_importer(parser);
    mark_import_meta_rsc_used(parser);

    let loc = parser.to_dependency_location(span.into());
    parser.add_dependency(Box::new(ImportMetaRscDependency::new_without_replacement(
      importer, loc,
    )));

    IMPORT_META_RSC_BINDING.to_string()
  }
}

fn get_rspack_rsc_importer(parser: &JavascriptParser) -> String {
  // Keep this aligned with RSC get_module_resource: path + query, no fragment.
  format!(
    "{}{}",
    parser.resource_data.path().map_or("", |path| path.as_str()),
    parser.resource_data.query().unwrap_or("")
  )
}

fn is_rsc_layer(parser: &JavascriptParser) -> bool {
  parser
    .get_module_layer()
    .is_some_and(|layer| layer == "react-server-components")
}

fn mark_import_meta_rsc_used(parser: &mut JavascriptParser) {
  match parser.build_info.rsc.as_mut() {
    Some(rsc) => {
      rsc.import_meta_rsc = true;
    }
    None => {
      parser.build_info.rsc = Some(RscMeta {
        module_type: RscModuleType::Server,
        server_refs: Default::default(),
        client_refs: Default::default(),
        import_meta_rsc: true,
        is_cjs: false,
        action_ids: Default::default(),
      });
    }
  }
}

#[rspack_macros::implemented_javascript_parser_hooks]
impl<'p, 'a> JavascriptParserPlugin<'p, 'a> for ImportMetaPlugin {
  fn evaluate_typeof(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: &'a UnaryExpr<'a>,
    for_name: &str,
  ) -> Option<eval::BasicEvaluatedExpression<'a>> {
    let evaluated = if for_name == expr_name::IMPORT_META {
      Some("object")
    } else if let Some(api) = import_meta_api_from_name(for_name) {
      if !(api.condition)(parser) {
        None
      } else {
        api.type_of
      }
    } else if let Some(member_expr) = expr.arg.as_member()
      && let Some(meta_expr) = member_expr.obj.as_meta_prop()
      && meta_expr
        .get_root_name()
        .is_some_and(|name| name == expr_name::IMPORT_META)
      && (match &member_expr.prop {
        MemberProp::Ident(_) => true,
        MemberProp::Computed(computed) => computed.expr.is_lit(),
        _ => false,
      })
    {
      Some("undefined")
    } else {
      None
    };
    evaluated
      .map(|e| eval::evaluate_to_string(e.to_string(), expr.span.real_lo(), expr.span.real_hi()))
  }

  fn evaluate_identifier(
    &self,
    parser: &mut JavascriptParser<'p>,
    for_name: &str,
    _member_expr_info: Option<&crate::visitors::ExpressionExpressionInfo>,
    start: u32,
    end: u32,
  ) -> Option<eval::BasicEvaluatedExpression<'p>> {
    let api = import_meta_api_from_name(for_name)?;
    if !(api.condition)(parser) {
      return None;
    }
    api
      .evaluate_identifier
      .map(|evaluate| evaluate(self, parser, start, end))
  }

  fn evaluate(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: &'a Expr,
  ) -> Option<eval::BasicEvaluatedExpression<'a>> {
    if let Some(member) = expr.as_member()
      && let Some(meta_prop) = member.obj.as_meta_prop()
      && meta_prop.kind == MetaPropKind::ImportMeta
    {
      if let Some(ident) = member.prop.as_ident() {
        if should_skip_import_meta_undefined_evaluate(parser, ident.sym.as_ref()) {
          return None;
        }
        let span = member.span();
        return Some(eval::evaluate_to_undefined(span.real_lo(), span.real_hi()));
      }
      if let Some(computed) = member.prop.as_computed()
        && computed.expr.is_lit()
      {
        // Check for computed properties like import.meta["dirname"]
        if let Some(str_lit) = computed.expr.as_lit().and_then(|lit| lit.as_str())
          && str_lit
            .value
            .as_str()
            .is_some_and(|value| should_skip_import_meta_undefined_evaluate(parser, value))
        {
          return None;
        }
        let span = member.span();
        return Some(eval::evaluate_to_undefined(span.real_lo(), span.real_hi()));
      }
    }
    None
  }

  fn r#typeof(
    &self,
    parser: &mut JavascriptParser<'p>,
    unary_expr: &UnaryExpr,
    for_name: &str,
  ) -> Option<bool> {
    let type_of = if for_name == expr_name::IMPORT_META {
      "object"
    } else {
      let api = import_meta_api_from_name(for_name)?;
      if !api.replace_typeof || !(api.condition)(parser) {
        return None;
      }
      api.type_of?
    };
    parser.add_presentational_dependency(Box::new(ConstDependency::new(
      unary_expr.span().into(),
      format!("'{type_of}'").into(),
    )));
    Some(true)
  }

  fn can_collect_destructuring_assignment_properties(
    &self,
    _parser: &mut JavascriptParser<'p>,
    expr: &Expr,
  ) -> Option<bool> {
    if expr.is_meta_prop() {
      return Some(true);
    }
    None
  }

  fn meta_property(
    &self,
    parser: &mut JavascriptParser<'p>,
    root_name: &swc_atoms::Atom,
    span: Span,
  ) -> Option<bool> {
    if root_name == expr_name::IMPORT_META {
      let destructuring_assignment_properties = parser
        .destructuring_assignment_properties
        .get(&span)
        .cloned();

      if let Some(referenced_properties_in_destructuring) = destructuring_assignment_properties {
        let mut content = vec![];
        for prop in referenced_properties_in_destructuring.iter() {
          let res = parser
            .plugin_drive
            .clone()
            .import_meta_property_in_destructuring(parser, prop);

          if let Some(property) = res {
            content.push(property);
            continue;
          }
          let destructuring = import_meta_api_from_property(prop.id.as_ref())
            .filter(|api| (api.condition)(parser))
            .and_then(|api| {
              api
                .destructuring
                .map(|destructuring| destructuring(self, parser, api.property, span))
            });
          content.push(destructuring.unwrap_or_else(|| {
            format!(
              r#"[{}]: {}"#,
              rspack_util::json_stringify_str(&prop.id),
              self.import_meta_unknown_property(&vec![prop.id.to_string()])
            )
          }));
        }
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          span.into(),
          format!("({{{}}})", content.join(",")).into(),
        )));
        Some(true)
      } else {
        // import.meta
        // warn when access import.meta directly
        let mut error: Error = create_traceable_error(
          "Critical dependency".into(),
          "Accessing import.meta directly is unsupported (only property access or destructuring is supported)".into(),
          parser.source.to_string(),
          span.into()
        );
        error.severity = Severity::Warning;
        parser.add_warning(error.into());

        let content = if parser.is_asi_position(span.start) {
          ";({})"
        } else {
          "({})"
        };
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          span.into(),
          content.into(),
        )));
        Some(true)
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
    let api = import_meta_api_from_name(for_name)?;
    if !(api.condition)(parser) {
      return None;
    }
    api.member(self, parser, member_expr)
  }

  fn call(
    &self,
    parser: &mut JavascriptParser<'p>,
    call_expr: &CallExpr,
    for_name: &str,
  ) -> Option<bool> {
    let api = import_meta_api_from_name(for_name)?;
    api
      .call
      .filter(|_| (api.condition)(parser))
      .map(|call| call(self, parser, call_expr))
  }

  fn unhandled_expression_member_chain(
    &self,
    parser: &mut JavascriptParser<'p>,
    root_info: &ExportedVariableInfo,
    expr: &MemberExpr,
  ) -> Option<bool> {
    match root_info {
      ExportedVariableInfo::Name(root) => {
        if root == expr_name::IMPORT_META {
          if matches!(self.0, ImportMeta::PreserveUnknown) {
            return Some(true);
          }
          let members = parser
            .get_member_expression_info(ExprRef::Member(expr), AllowedMemberTypes::Expression)
            .and_then(|info| match info {
              MemberExpressionInfo::Expression(res) => Some(res),
              _ => None,
            });

          let dep = if let Some(members) = members {
            if members.members.get(1).is_some()
              && members
                .members_optionals
                .get(1)
                .is_some_and(|optional| *optional)
            {
              ConstDependency::new(expr.span().into(), "undefined".into())
            } else {
              ConstDependency::new(
                expr.span().into(),
                self
                  .import_meta_unknown_property(
                    &members.members.iter().map(|x| x.to_string()).collect_vec(),
                  )
                  .into(),
              )
            }
          } else {
            ConstDependency::new(expr.span().into(), "undefined".into())
          };

          parser.add_presentational_dependency(Box::new(dep));
          return Some(true);
        }
      }
      ExportedVariableInfo::VariableInfo(_) => (),
    }
    None
  }
}

// use when parser.import_meta is false
pub struct ImportMetaDisabledPlugin;

#[rspack_macros::implemented_javascript_parser_hooks]
impl<'p, 'a> JavascriptParserPlugin<'p, 'a> for ImportMetaDisabledPlugin {
  fn meta_property(
    &self,
    parser: &mut JavascriptParser<'p>,
    root_name: &swc_atoms::Atom,
    span: Span,
  ) -> Option<bool> {
    let import_meta_name = parser.compiler_options.output.import_meta_name.clone();
    if import_meta_name == expr_name::IMPORT_META {
      None
    } else if root_name == expr_name::IMPORT_META {
      parser.add_presentational_dependency(Box::new(ConstDependency::new(
        span.into(),
        import_meta_name.into(),
      )));
      Some(true)
    } else {
      None
    }
  }
}
