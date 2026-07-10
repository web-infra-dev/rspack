use concat_string::concat_string;
use itertools::Itertools;
use rspack_core::{
  ArcComputed, ConstDependency, ContextDependency, ContextMode, ContextOptions, DependencyCategory,
  DependencyRange, ImportMeta, ImportMetaKnownProperties, ResolvedModuleOptions, RscMeta,
  RscModuleType, RuntimeGlobals, RuntimeRequirementsDependency, property_access,
};
use rspack_error::{Error, Severity};
use rspack_util::SpanExt;
use swc_atoms::Atom;
use swc_experimental_ecma_ast::{
  AssignExpr, CallExpr, Expr, GetSpan, MemberExpr, MemberProp, MetaPropKind, Span, UnaryExpr,
};
use url::Url;

use super::{
  JavascriptParserPlugin,
  api_plugin::{
    ImportMetaRuntimeApi, import_meta_runtime_api_assign, import_meta_runtime_api_call,
    import_meta_runtime_api_from_name, import_meta_runtime_api_from_property,
    import_meta_runtime_api_member, is_simple_assign_op,
    render_import_meta_runtime_api_destructuring,
  },
  import_meta_path::{
    get_import_meta_eval_value, get_import_meta_member_replacement, should_handle_import_meta_path,
  },
};
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

#[derive(Clone, Copy)]
struct ImportMetaBuiltinProperty {
  name: &'static str,
  property: ImportMetaKnownProperties,
  type_of: &'static str,
}

static IMPORT_META_BUILTIN_PROPERTIES: &[ImportMetaBuiltinProperty] = &[
  ImportMetaBuiltinProperty {
    name: expr_name::IMPORT_META_URL,
    property: ImportMetaKnownProperties::URL,
    type_of: "string",
  },
  ImportMetaBuiltinProperty {
    name: expr_name::IMPORT_META_RESOLVE,
    property: ImportMetaKnownProperties::RESOLVE,
    type_of: "function",
  },
  ImportMetaBuiltinProperty {
    name: expr_name::IMPORT_META_VERSION,
    property: ImportMetaKnownProperties::WEBPACK,
    type_of: "number",
  },
  ImportMetaBuiltinProperty {
    name: expr_name::IMPORT_META_MAIN,
    property: ImportMetaKnownProperties::MAIN,
    type_of: "boolean",
  },
  ImportMetaBuiltinProperty {
    name: expr_name::IMPORT_META_FILENAME,
    property: ImportMetaKnownProperties::FILENAME,
    type_of: "string",
  },
  ImportMetaBuiltinProperty {
    name: expr_name::IMPORT_META_DIRNAME,
    property: ImportMetaKnownProperties::DIRNAME,
    type_of: "string",
  },
  ImportMetaBuiltinProperty {
    name: expr_name::IMPORT_META_RSPACK_RSC,
    property: ImportMetaKnownProperties::RSPACK_RSC,
    type_of: "object",
  },
];

impl ImportMetaBuiltinProperty {
  fn from_name(name: &str) -> Option<&'static Self> {
    IMPORT_META_BUILTIN_PROPERTIES
      .iter()
      .find(|property| property.name == name)
  }

  fn from_property(property: &str) -> Option<&'static Self> {
    IMPORT_META_BUILTIN_PROPERTIES
      .iter()
      .find(|builtin| builtin.property_name() == property)
  }

  fn property_name(&self) -> &'static str {
    debug_assert!(self.name.starts_with(expr_name::IMPORT_META_PREFIX));
    &self.name[expr_name::IMPORT_META_PREFIX.len()..]
  }

  fn enabled(&self, plugin: &ImportMetaPlugin, parser: &JavascriptParser) -> bool {
    if !plugin.known_property_enabled(self.property) {
      return false;
    }

    match self.property {
      ImportMetaKnownProperties::RESOLVE => {
        parser.javascript_options.import_meta_resolve == Some(true)
      }
      ImportMetaKnownProperties::RSPACK_RSC => is_rsc_layer(parser),
      _ => true,
    }
  }

  fn evaluate_typeof(&self, plugin: &ImportMetaPlugin, parser: &JavascriptParser) -> Option<&str> {
    if matches!(
      self.property,
      ImportMetaKnownProperties::FILENAME | ImportMetaKnownProperties::DIRNAME
    ) && !should_handle_import_meta_path(parser, self.property)
    {
      return None;
    }
    self.enabled(plugin, parser).then_some(self.type_of)
  }

  fn evaluate_identifier<'p>(
    &self,
    plugin: &ImportMetaPlugin,
    parser: &mut JavascriptParser<'p>,
    start: u32,
    end: u32,
  ) -> Option<eval::BasicEvaluatedExpression<'p>> {
    if !self.enabled(plugin, parser) {
      return None;
    }

    match self.property {
      ImportMetaKnownProperties::URL => Some(eval::evaluate_to_string(
        plugin.import_meta_url(parser),
        start,
        end,
      )),
      ImportMetaKnownProperties::RESOLVE => Some(eval::evaluate_to_identifier(
        expr_name::IMPORT_META_RESOLVE.into(),
        expr_name::IMPORT_META_RESOLVE.into(),
        Some(true),
        start,
        end,
      )),
      ImportMetaKnownProperties::WEBPACK => Some(eval::evaluate_to_number(5_f64, start, end)),
      ImportMetaKnownProperties::FILENAME | ImportMetaKnownProperties::DIRNAME => {
        get_import_meta_eval_value(parser, self.property)
          .map(|value| eval::evaluate_to_string(value, start, end))
      }
      ImportMetaKnownProperties::MAIN | ImportMetaKnownProperties::RSPACK_RSC => None,
      _ => unreachable!("unexpected import.meta builtin property"),
    }
  }

  fn add_typeof_dependency(
    &self,
    plugin: &ImportMetaPlugin,
    parser: &mut JavascriptParser,
    unary_expr: &UnaryExpr,
  ) -> Option<bool> {
    let type_of = self.evaluate_typeof(plugin, parser)?;
    parser.add_presentational_dependency(Box::new(ConstDependency::new(
      unary_expr.span().into(),
      concat_string!("'", type_of, "'").into(),
    )));
    Some(true)
  }

  fn render_destructuring(
    &self,
    plugin: &ImportMetaPlugin,
    parser: &mut JavascriptParser,
    span: Span,
  ) -> Option<String> {
    if !self.enabled(plugin, parser) {
      return None;
    }

    let property = self.property_name();
    match self.property {
      ImportMetaKnownProperties::URL => Some(concat_string!(
        property,
        ": \"",
        plugin.import_meta_url(parser),
        "\""
      )),
      ImportMetaKnownProperties::WEBPACK => {
        Some(concat_string!(property, ": ", plugin.import_meta_version()))
      }
      ImportMetaKnownProperties::MAIN => Some(concat_string!(
        property,
        ": ",
        plugin.import_meta_main(parser)
      )),
      ImportMetaKnownProperties::FILENAME | ImportMetaKnownProperties::DIRNAME => {
        get_import_meta_member_replacement(parser, self.property)
          .map(|value| concat_string!(property, ": ", value))
      }
      ImportMetaKnownProperties::RSPACK_RSC => Some(concat_string!(
        property,
        ": ",
        plugin.process_rspack_rsc_destructuring(parser, span)
      )),
      ImportMetaKnownProperties::RESOLVE => None,
      _ => unreachable!("unexpected import.meta builtin property"),
    }
  }

  fn member(
    &self,
    plugin: &ImportMetaPlugin,
    parser: &mut JavascriptParser,
    member_expr: &MemberExpr,
  ) -> Option<bool> {
    if !self.enabled(plugin, parser) {
      return None;
    }

    let replacement = match self.property {
      ImportMetaKnownProperties::URL => concat_string!("'", plugin.import_meta_url(parser), "'"),
      ImportMetaKnownProperties::WEBPACK => plugin.import_meta_version(),
      ImportMetaKnownProperties::MAIN => plugin.import_meta_main(parser),
      ImportMetaKnownProperties::FILENAME | ImportMetaKnownProperties::DIRNAME => {
        get_import_meta_member_replacement(parser, self.property)?
      }
      ImportMetaKnownProperties::RSPACK_RSC => {
        plugin.process_rspack_rsc(parser, member_expr);
        return Some(true);
      }
      ImportMetaKnownProperties::RESOLVE => return None,
      _ => unreachable!("unexpected import.meta builtin property"),
    };

    parser.add_presentational_dependency(Box::new(ConstDependency::new(
      member_expr.span().into(),
      replacement.into(),
    )));
    Some(true)
  }

  fn skip_undefined_evaluation(
    &self,
    plugin: &ImportMetaPlugin,
    parser: &JavascriptParser,
  ) -> bool {
    match self.property {
      // dirname/filename may be preserved at runtime based on node options, so don't fold them to undefined.
      ImportMetaKnownProperties::FILENAME | ImportMetaKnownProperties::DIRNAME => true,
      ImportMetaKnownProperties::MAIN | ImportMetaKnownProperties::RSPACK_RSC => {
        self.enabled(plugin, parser)
      }
      _ => false,
    }
  }
}

pub struct ImportMetaPlugin(pub(crate) ArcComputed<ResolvedModuleOptions, ImportMeta>);

impl ImportMetaPlugin {
  fn known_property_from_name(name: &str) -> Option<ImportMetaKnownProperties> {
    if let Some(property) = ImportMetaBuiltinProperty::from_name(name) {
      return Some(property.property);
    }
    if let Some(api) = import_meta_runtime_api_from_name(name) {
      return Some(api.property);
    }
    None
  }

  fn preserve_property(&self, property: Option<&str>) -> bool {
    match self.0.as_ref() {
      ImportMeta::PreserveUnknown => true,
      ImportMeta::Granular(_) => property.is_none_or(|property| {
        let name = concat_string!(expr_name::IMPORT_META, ".", property);
        Self::known_property_from_name(&name)
          .is_none_or(|property| !self.known_property_enabled(property))
      }),
      ImportMeta::Enabled | ImportMeta::Disabled => false,
    }
  }

  fn known_property_enabled(&self, property: ImportMetaKnownProperties) -> bool {
    self.0.is_known_property_enabled(property)
  }

  fn runtime_api_enabled(&self, api: &ImportMetaRuntimeApi) -> bool {
    self.known_property_enabled(api.property)
  }

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
    concat_string!(
      "(",
      parser.parser_runtime_requirements.module_cache,
      "[",
      parser.parser_runtime_requirements.entry_module_id,
      "] === ",
      parser
        .parser_runtime_requirements
        .module_argument(&parser.build_info.module_argument),
      ")"
    )
  }

  fn import_meta_unknown_property(&self, members: &Vec<String>) -> String {
    if self.preserve_property(members.first().map(|property| property.as_str())) {
      concat_string!("import.meta", property_access(members, 0))
    } else {
      concat_string!(
        "/* unsupported import.meta.",
        members.join("."),
        " */ undefined",
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
    let mut evaluated = None;
    if for_name == expr_name::IMPORT_META {
      evaluated = Some("object".to_string());
    } else if let Some(property) = ImportMetaBuiltinProperty::from_name(for_name)
      && let Some(type_of) = property.evaluate_typeof(self, parser)
    {
      evaluated = Some(type_of.to_string())
    } else if let Some(api) = import_meta_runtime_api_from_name(for_name)
      && self.runtime_api_enabled(api)
    {
      evaluated = Some(api.type_of.to_string())
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
      && member_expr
        .prop
        .as_ident()
        .map(|ident| !self.preserve_property(Some(ident.sym.as_ref())))
        .or_else(|| {
          member_expr
            .prop
            .as_computed()
            .and_then(|computed| computed.expr.as_lit())
            .and_then(|lit| lit.as_str())
            .and_then(|str_lit| str_lit.value.as_str())
            .map(|value| !self.preserve_property(Some(value)))
        })
        .unwrap_or(false)
    {
      evaluated = Some("undefined".to_string())
    }
    evaluated.map(|e| eval::evaluate_to_string(e, expr.span.real_lo(), expr.span.real_hi()))
  }

  fn evaluate_identifier(
    &self,
    parser: &mut JavascriptParser<'p>,
    for_name: &str,
    _member_expr_info: Option<&crate::visitors::ExpressionExpressionInfo>,
    start: u32,
    end: u32,
  ) -> Option<eval::BasicEvaluatedExpression<'p>> {
    let property = ImportMetaBuiltinProperty::from_name(for_name)?;
    property.evaluate_identifier(self, parser, start, end)
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
        // - Skip `dirname` and `filename` - they are handled by `member`
        //   and may have runtime values when node.__dirname/node.__filename is false
        // - Skip `main` - it will generate dynamic code: `moduleCache[entryModuleId] === module`
        if ImportMetaBuiltinProperty::from_property(ident.sym.as_ref())
          .is_some_and(|property| property.skip_undefined_evaluation(self, parser))
          || import_meta_runtime_api_from_property(ident.sym.as_ref())
            .is_some_and(|api| self.runtime_api_enabled(api))
          || self.preserve_property(Some(ident.sym.as_ref()))
        {
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
          && str_lit.value.as_str().is_some_and(|value| {
            ImportMetaBuiltinProperty::from_property(value)
              .is_some_and(|property| property.skip_undefined_evaluation(self, parser))
              || import_meta_runtime_api_from_property(value)
                .is_some_and(|api| self.runtime_api_enabled(api))
              || self.preserve_property(Some(value))
          })
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
    match for_name {
      expr_name::IMPORT_META => {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          unary_expr.span().into(),
          "'object'".into(),
        )));
        Some(true)
      }
      _ => {
        if let Some(property) = ImportMetaBuiltinProperty::from_name(for_name) {
          return property.add_typeof_dependency(self, parser, unary_expr);
        }
        let api = import_meta_runtime_api_from_name(for_name)?;
        if !self.runtime_api_enabled(api) {
          return None;
        }
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          unary_expr.span().into(),
          format!("'{}'", api.type_of).into(),
        )));
        Some(true)
      }
    }
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
        let mut content = String::from("({");
        for (index, prop) in referenced_properties_in_destructuring.iter().enumerate() {
          if index > 0 {
            content.push(',');
          }
          let res = parser
            .plugin_drive
            .clone()
            .import_meta_property_in_destructuring(parser, prop);

          if let Some(property) = res {
            content.push_str(&property);
            continue;
          }
          if let Some(property) = ImportMetaBuiltinProperty::from_property(prop.id.as_ref()) {
            if let Some(rendered) = property.render_destructuring(self, parser, span) {
              content.push_str(&rendered);
            } else {
              content.push('[');
              content.push_str(&rspack_util::json_stringify_str(&prop.id));
              content.push_str("]: ");
              content.push_str(&self.import_meta_unknown_property(&vec![prop.id.to_string()]));
            }
          } else if let Some(api) = import_meta_runtime_api_from_property(prop.id.as_ref()) {
            if self.runtime_api_enabled(api)
              && let Some(property) = render_import_meta_runtime_api_destructuring(parser, api)
            {
              content.push_str(&property);
            } else {
              content.push('[');
              content.push_str(&rspack_util::json_stringify_str(&prop.id));
              content.push_str("]: ");
              content.push_str(&self.import_meta_unknown_property(&vec![prop.id.to_string()]));
            }
          } else {
            content.push('[');
            content.push_str(&rspack_util::json_stringify_str(&prop.id));
            content.push_str("]: ");
            content.push_str(&self.import_meta_unknown_property(&vec![prop.id.to_string()]));
          }
        }
        content.push_str("})");
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          span.into(),
          content.into(),
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
    if let Some(property) = ImportMetaBuiltinProperty::from_name(for_name)
      && let Some(handled) = property.member(self, parser, member_expr)
    {
      Some(handled)
    } else if let Some(api) = import_meta_runtime_api_from_name(for_name)
      && self.runtime_api_enabled(api)
    {
      import_meta_runtime_api_member(parser, member_expr.span(), api)
    } else {
      None
    }
  }

  fn call(
    &self,
    parser: &mut JavascriptParser<'p>,
    call_expr: &CallExpr,
    for_name: &str,
  ) -> Option<bool> {
    if parser.javascript_options.import_meta_resolve == Some(true)
      && for_name == expr_name::IMPORT_META_RESOLVE
      && self.known_property_enabled(ImportMetaKnownProperties::RESOLVE)
    {
      self.process_import_meta_resolve(parser, call_expr);
      return Some(true);
    }
    if let Some(api) = import_meta_runtime_api_from_name(for_name) {
      if !self.runtime_api_enabled(api) {
        return None;
      }
      return import_meta_runtime_api_call(parser, call_expr, api);
    }
    None
  }

  fn assign_member_chain(
    &self,
    parser: &mut JavascriptParser,
    expr: &AssignExpr,
    members: &[Atom],
    member_ranges: &[Span],
    for_name: &str,
  ) -> Option<bool> {
    if for_name != expr_name::IMPORT_META {
      return None;
    }
    let property = members.first()?;
    let api = import_meta_runtime_api_from_property(property.as_ref())?;
    if !self.runtime_api_enabled(api) {
      return None;
    }
    let full_assignment = members.len() == 1;
    let span = if full_assignment {
      expr.left.span()
    } else {
      member_ranges
        .get(1)
        .copied()
        .unwrap_or_else(|| expr.left.span())
    };
    let handled = import_meta_runtime_api_assign(
      parser,
      span,
      api,
      full_assignment,
      is_simple_assign_op(expr.op),
    );
    if handled.is_some() {
      parser.walk_expression(&expr.right);
    }
    handled
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
          if matches!(self.0.as_ref(), ImportMeta::PreserveUnknown) {
            return Some(true);
          }
          let members = parser
            .get_member_expression_info(ExprRef::Member(expr), AllowedMemberTypes::Expression)
            .and_then(|info| match info {
              MemberExpressionInfo::Expression(res) => Some(res),
              _ => None,
            });

          let dep = if let Some(members) = members {
            if self.preserve_property(members.members.first().map(|property| property.as_str())) {
              return Some(true);
            }
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
