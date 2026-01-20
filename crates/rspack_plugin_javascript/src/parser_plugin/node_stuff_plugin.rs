use rspack_core::{
  CachedConstDependency, ConstDependency, ImportMeta, NodeDirnameOption, NodeFilenameOption,
  NodeGlobalOption, RuntimeGlobals, RuntimeRequirementsDependency, get_context, parse_resource,
};
use rspack_error::{Diagnostic, cyan, yellow};
use rspack_util::SpanExt;
use sugar_path::SugarPath;
use swc_core::{common::Spanned, ecma::ast::Expr};
use url::Url;

use crate::{
  JavascriptParserPlugin,
  dependency::ExternalModuleDependency,
  utils::eval,
  visitors::{DestructuringAssignmentProperty, JavascriptParser},
};

const DIRNAME: &str = "__dirname";
const FILENAME: &str = "__filename";
const IMPORT_META_DIRNAME: &str = "import.meta.dirname";
const IMPORT_META_FILENAME: &str = "import.meta.filename";
const GLOBAL: &str = "global";
const MOCK_DIRNAME: &str = "/";
const MOCK_FILENAME: &str = "/index.js";

/// Represents the type of import.meta property being handled (filename or dirname)
#[derive(Clone, Copy)]
enum NodeMetaProperty {
  Filename,
  Dirname,
}

impl NodeMetaProperty {
  /// Returns the mock value for this property
  fn mock_value(&self) -> &'static str {
    match self {
      NodeMetaProperty::Filename => MOCK_FILENAME,
      NodeMetaProperty::Dirname => MOCK_DIRNAME,
    }
  }

  /// Returns the import.meta property name
  fn import_meta_name(&self) -> &'static str {
    match self {
      NodeMetaProperty::Filename => "import.meta.filename",
      NodeMetaProperty::Dirname => "import.meta.dirname",
    }
  }

  /// Returns the CJS equivalent variable name
  fn cjs_name(&self) -> &'static str {
    match self {
      NodeMetaProperty::Filename => "__filename",
      NodeMetaProperty::Dirname => "__dirname",
    }
  }

  /// Returns the warning code for this property
  fn warning_code(&self) -> &'static str {
    match self {
      NodeMetaProperty::Filename => "NODE_IMPORT_META_FILENAME",
      NodeMetaProperty::Dirname => "NODE_IMPORT_META_DIRNAME",
    }
  }

  /// Returns the warning message for this property
  fn warning_message(&self) -> String {
    match self {
      NodeMetaProperty::Filename => format!(
        "\"{}\" is used and has been mocked. Remove it from your code, or set `{}` to disable this warning.",
        yellow(&IMPORT_META_FILENAME),
        cyan(&"node.__filename")
      ),
      NodeMetaProperty::Dirname => format!(
        "\"{}\" is used and has been mocked. Remove it from your code, or set `{}` to disable this warning.",
        yellow(&IMPORT_META_DIRNAME),
        cyan(&"node.__dirname")
      ),
    }
  }

  /// Returns the runtime expression for NodeModule mode
  fn node_module_runtime_expr(&self) -> &'static str {
    match self {
      NodeMetaProperty::Filename => "__rspack_fileURLToPath(import.meta.url)",
      NodeMetaProperty::Dirname => "__rspack_dirname(__rspack_fileURLToPath(import.meta.url))",
    }
  }

  /// Returns whether dirname is disabled based on node options
  fn is_disabled(&self, node_option: &rspack_core::NodeOption) -> bool {
    match self {
      NodeMetaProperty::Filename => matches!(node_option.filename, NodeFilenameOption::False),
      NodeMetaProperty::Dirname => matches!(node_option.dirname, NodeDirnameOption::False),
    }
  }

  /// Check if the option is Mock
  fn is_mock(&self, node_option: &rspack_core::NodeOption) -> bool {
    match self {
      NodeMetaProperty::Filename => matches!(node_option.filename, NodeFilenameOption::Mock),
      NodeMetaProperty::Dirname => matches!(node_option.dirname, NodeDirnameOption::Mock),
    }
  }

  /// Check if the option is WarnMock
  fn is_warn_mock(&self, node_option: &rspack_core::NodeOption) -> bool {
    match self {
      NodeMetaProperty::Filename => matches!(node_option.filename, NodeFilenameOption::WarnMock),
      NodeMetaProperty::Dirname => matches!(node_option.dirname, NodeDirnameOption::WarnMock),
    }
  }

  /// Check if the option is True
  fn is_true(&self, node_option: &rspack_core::NodeOption) -> bool {
    match self {
      NodeMetaProperty::Filename => matches!(node_option.filename, NodeFilenameOption::True),
      NodeMetaProperty::Dirname => matches!(node_option.dirname, NodeDirnameOption::True),
    }
  }

  /// Check if the option is EvalOnly
  fn is_eval_only(&self, node_option: &rspack_core::NodeOption) -> bool {
    match self {
      NodeMetaProperty::Filename => matches!(node_option.filename, NodeFilenameOption::EvalOnly),
      NodeMetaProperty::Dirname => matches!(node_option.dirname, NodeDirnameOption::EvalOnly),
    }
  }

  /// Check if the option is NodeModule
  fn is_node_module(&self, node_option: &rspack_core::NodeOption) -> bool {
    match self {
      NodeMetaProperty::Filename => matches!(node_option.filename, NodeFilenameOption::NodeModule),
      NodeMetaProperty::Dirname => matches!(node_option.dirname, NodeDirnameOption::NodeModule),
    }
  }
}

/// Plugin for handling Node.js-specific variables like `__dirname`, `__filename`, `global`,
/// and their ESM equivalents `import.meta.dirname` and `import.meta.filename`.
///
/// This mirrors webpack's approach where NodeStuffPlugin is registered once per module type
/// with boolean flags controlling which features to handle:
/// - `handle_cjs`: handle `__dirname`, `__filename`, `global`
/// - `handle_esm`: handle `import.meta.dirname`, `import.meta.filename`
pub struct NodeStuffPlugin {
  /// When true, handle __dirname/__filename/global (CJS features)
  handle_cjs: bool,
  /// When true, handle import.meta.dirname/filename (ESM features)
  handle_esm: bool,
}

impl NodeStuffPlugin {
  pub fn new(handle_cjs: bool, handle_esm: bool) -> Self {
    Self {
      handle_cjs,
      handle_esm,
    }
  }

  /// Get the relative path value for the given property (filename or dirname)
  fn get_relative_path(parser: &JavascriptParser, property: NodeMetaProperty) -> Option<String> {
    match property {
      NodeMetaProperty::Filename => Some(
        parser
          .resource_data
          .path()?
          .as_std_path()
          .relative(&parser.compiler_options.context)
          .to_string_lossy()
          .to_string(),
      ),
      NodeMetaProperty::Dirname => Some(
        parser
          .resource_data
          .path()?
          .parent()?
          .as_std_path()
          .relative(&parser.compiler_options.context)
          .to_string_lossy()
          .to_string(),
      ),
    }
  }

  /// Get the absolute path value for node-module mode
  fn get_absolute_path(parser: &JavascriptParser, property: NodeMetaProperty) -> Option<String> {
    let path = Url::from_file_path(parser.resource_data.resource())
      .expect("should be a path")
      .to_file_path()
      .expect("should be a path");

    match property {
      NodeMetaProperty::Filename => Some(path.to_string_lossy().into_owned()),
      NodeMetaProperty::Dirname => Some(
        path
          .parent()
          .expect("should have a parent")
          .to_string_lossy()
          .into_owned(),
      ),
    }
  }

  /// Get the eval-only path value
  fn get_eval_only_path(parser: &JavascriptParser, property: NodeMetaProperty) -> Option<String> {
    match property {
      NodeMetaProperty::Filename => {
        let resource = parse_resource(parser.resource_data.path()?.as_str())?;
        Some(resource.path.to_string())
      }
      NodeMetaProperty::Dirname => Some(parser.resource_data.path()?.parent()?.to_string()),
    }
  }

  /// Add external dependencies for NodeModule mode
  fn add_node_module_dependencies(parser: &mut JavascriptParser, property: NodeMetaProperty) {
    let external_url_dep = ExternalModuleDependency::new(
      "url".to_string(),
      vec![(
        "fileURLToPath".to_string(),
        "__rspack_fileURLToPath".to_string(),
      )],
      None,
    );
    parser.add_presentational_dependency(Box::new(external_url_dep));

    if matches!(property, NodeMetaProperty::Dirname) {
      let external_path_dep = ExternalModuleDependency::new(
        "path".to_string(),
        vec![("dirname".to_string(), "__rspack_dirname".to_string())],
        None,
      );
      parser.add_presentational_dependency(Box::new(external_path_dep));
    }
  }

  fn add_cjs_node_module_dependency(
    parser: &mut JavascriptParser,
    ident_span: swc_core::common::Span,
    name: &str,
    property: NodeMetaProperty,
  ) {
    Self::add_node_module_dependencies(parser, property);
    let const_dep = CachedConstDependency::new(
      ident_span.into(),
      name.into(),
      property.node_module_runtime_expr().into(),
    );
    parser.add_presentational_dependency(Box::new(const_dep));
  }

  /// Get the evaluated value for import.meta.filename/dirname
  fn get_import_meta_eval_value(
    parser: &JavascriptParser,
    property: NodeMetaProperty,
  ) -> Option<String> {
    let node_option = parser.compiler_options.node.as_ref()?;

    if property.is_disabled(node_option) {
      return None;
    }

    if property.is_mock(node_option) || property.is_warn_mock(node_option) {
      return Some(property.mock_value().to_string());
    }

    if property.is_true(node_option) {
      return Self::get_relative_path(parser, property);
    }

    if property.is_eval_only(node_option) {
      return Self::get_eval_only_path(parser, property);
    }

    if property.is_node_module(node_option) {
      return Self::get_absolute_path(parser, property);
    }

    None
  }

  /// Get the member replacement value for import.meta.filename/dirname
  fn get_import_meta_member_replacement(
    parser: &mut JavascriptParser,
    property: NodeMetaProperty,
  ) -> Option<String> {
    let node_option = match parser.compiler_options.node.as_ref() {
      None => return Some(property.import_meta_name().to_string()),
      Some(opt) => opt,
    };

    if property.is_disabled(node_option) {
      return Some(property.import_meta_name().to_string());
    }

    if property.is_mock(node_option) {
      return Some(format!("'{}'", property.mock_value()));
    }

    if property.is_warn_mock(node_option) {
      parser.add_warning(Diagnostic::warn(
        property.warning_code().to_string(),
        property.warning_message(),
      ));
      return Some(format!("'{}'", property.mock_value()));
    }

    if property.is_true(node_option) {
      let path = Self::get_relative_path(parser, property)?;
      return Some(format!("'{path}'"));
    }

    if property.is_eval_only(node_option) {
      return Some(if parser.compiler_options.output.module {
        property.import_meta_name().to_string()
      } else {
        property.cjs_name().to_string()
      });
    }

    if property.is_node_module(node_option) {
      // Check if native import.meta.dirname/filename is supported
      if parser.compiler_options.output.module
        && parser
          .compiler_options
          .output
          .environment
          .supports_import_meta_dirname_and_filename()
      {
        // Keep as import.meta.filename/dirname - runtime supports it
        return Some(property.import_meta_name().to_string());
      }
      Self::add_node_module_dependencies(parser, property);
      return Some(property.node_module_runtime_expr().to_string());
    }

    None
  }

  /// Get the destructuring replacement value for import.meta.filename/dirname
  fn get_import_meta_destructuring_value(
    parser: &mut JavascriptParser,
    property: NodeMetaProperty,
  ) -> Option<String> {
    let node_option = match parser.compiler_options.node.as_ref() {
      None => return Some(property.import_meta_name().to_string()),
      Some(opt) => opt,
    };

    if property.is_disabled(node_option) {
      return Some(property.import_meta_name().to_string());
    }

    if property.is_mock(node_option) {
      return Some(format!("\"{}\"", property.mock_value()));
    }

    if property.is_warn_mock(node_option) {
      parser.add_warning(Diagnostic::warn(
        property.warning_code().to_string(),
        property.warning_message(),
      ));
      return Some(format!("\"{}\"", property.mock_value()));
    }

    if property.is_true(node_option) {
      let path = Self::get_relative_path(parser, property)?;
      return Some(format!("\"{path}\""));
    }

    if property.is_eval_only(node_option) {
      return Some(if parser.compiler_options.output.module {
        property.import_meta_name().to_string()
      } else {
        property.cjs_name().to_string()
      });
    }

    if property.is_node_module(node_option) {
      // Check if native import.meta.dirname/filename is supported
      if parser.compiler_options.output.module
        && parser
          .compiler_options
          .output
          .environment
          .supports_import_meta_dirname_and_filename()
      {
        return Some(property.import_meta_name().to_string());
      }
      Self::add_node_module_dependencies(parser, property);
      return Some(property.node_module_runtime_expr().to_string());
    }

    None
  }
}

impl JavascriptParserPlugin for NodeStuffPlugin {
  fn identifier(
    &self,
    parser: &mut JavascriptParser,
    ident: &swc_core::ecma::ast::Ident,
    for_name: &str,
  ) -> Option<bool> {
    // Skip CJS handling if not enabled
    if !self.handle_cjs {
      return None;
    }

    let Some(node_option) = parser.compiler_options.node.as_ref() else {
      // When node: false, this plugin is not registered for CJS modules
      return None;
    };
    if for_name == DIRNAME {
      let dirname = match node_option.dirname {
        NodeDirnameOption::Mock => Some(MOCK_DIRNAME.to_string()),
        NodeDirnameOption::WarnMock => {
          parser.add_warning(Diagnostic::warn(
            "NODE_DIRNAME".to_string(),
            format!("\"{}\" is used and has been mocked. Remove it from your code, or set `{}` to disable this warning.", yellow(&DIRNAME), cyan(&"node.__dirname")),
          ));
          Some(MOCK_DIRNAME.to_string())
        }
        NodeDirnameOption::NodeModule => {
          // `ExternalModuleDependency` extends `CachedConstDependency` in webpack.
          // We need to create two separate dependencies in Rspack.
          Self::add_cjs_node_module_dependency(
            parser,
            ident.span,
            DIRNAME,
            NodeMetaProperty::Dirname,
          );
          return Some(true);
        }
        NodeDirnameOption::EvalOnly => {
          // For CJS output, preserve __dirname (let Node.js runtime handle it)
          if !parser.compiler_options.output.module {
            return None;
          }
          Self::add_cjs_node_module_dependency(
            parser,
            ident.span,
            DIRNAME,
            NodeMetaProperty::Dirname,
          );
          return Some(true);
        }
        NodeDirnameOption::True => Some(
          parser
            .resource_data
            .path()?
            .parent()?
            .as_std_path()
            .relative(&parser.compiler_options.context)
            .to_string_lossy()
            .to_string(),
        ),
        NodeDirnameOption::False => None,
      };
      if let Some(dirname) = dirname {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          ident.span.into(),
          serde_json::to_string(&dirname)
            .expect("should render dirname")
            .into(),
        )));
        return Some(true);
      }
    } else if for_name == FILENAME {
      let filename = match node_option.filename {
        NodeFilenameOption::Mock => Some(MOCK_FILENAME.to_string()),
        NodeFilenameOption::WarnMock => {
          parser.add_warning(Diagnostic::warn(
            "NODE_FILENAME".to_string(),
            format!("\"{}\" is used and has been mocked. Remove it from your code, or set `{}` to disable this warning.", yellow(&FILENAME), cyan(&"node.__filename")),
          ));
          Some(MOCK_FILENAME.to_string())
        }
        NodeFilenameOption::NodeModule => {
          // `ExternalModuleDependency` extends `CachedConstDependency` in webpack.
          // We need to create two separate dependencies in Rspack.
          Self::add_cjs_node_module_dependency(
            parser,
            ident.span,
            FILENAME,
            NodeMetaProperty::Filename,
          );
          return Some(true);
        }
        NodeFilenameOption::EvalOnly => {
          // For CJS output, preserve __filename (let Node.js runtime handle it)
          if !parser.compiler_options.output.module {
            return None;
          }
          Self::add_cjs_node_module_dependency(
            parser,
            ident.span,
            FILENAME,
            NodeMetaProperty::Filename,
          );
          return Some(true);
        }
        NodeFilenameOption::True => Some(
          parser
            .resource_data
            .path()?
            .as_std_path()
            .relative(&parser.compiler_options.context)
            .to_string_lossy()
            .to_string(),
        ),
        NodeFilenameOption::False => None,
      };
      if let Some(filename) = filename {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          ident.span.into(),
          serde_json::to_string(&filename)
            .expect("should render filename")
            .into(),
        )));
        return Some(true);
      }
    } else if for_name == GLOBAL
      && matches!(
        node_option.global,
        NodeGlobalOption::True | NodeGlobalOption::Warn
      )
    {
      parser.add_presentational_dependency(Box::new(RuntimeRequirementsDependency::new(
        ident.span.into(),
        RuntimeGlobals::GLOBAL,
      )));
      return Some(true);
    }
    None
  }

  fn rename(&self, parser: &mut JavascriptParser, expr: &Expr, for_name: &str) -> Option<bool> {
    // Skip CJS handling if not enabled
    if !self.handle_cjs {
      return None;
    }

    let node_option = parser.compiler_options.node.as_ref()?;
    if for_name == GLOBAL
      && matches!(
        node_option.global,
        NodeGlobalOption::True | NodeGlobalOption::Warn
      )
    {
      parser.add_presentational_dependency(Box::new(RuntimeRequirementsDependency::new(
        expr.span().into(),
        RuntimeGlobals::GLOBAL,
      )));
      return Some(false);
    }
    None
  }

  fn r#typeof(
    &self,
    parser: &mut JavascriptParser,
    unary_expr: &swc_core::ecma::ast::UnaryExpr,
    for_name: &str,
  ) -> Option<bool> {
    use crate::visitors::expr_name;

    match for_name {
      FILENAME => {
        // Skip CJS __filename if not handling CJS
        if !self.handle_cjs {
          return None;
        }
        // Skip if node: false or node.filename is disabled
        if parser.compiler_options.node.is_none()
          || parser
            .compiler_options
            .node
            .as_ref()
            .is_some_and(|node_option| matches!(node_option.filename, NodeFilenameOption::False))
        {
          return None;
        }
      }
      expr_name::IMPORT_META_FILENAME => {
        // Skip ESM import.meta.filename if not handling ESM
        if !self.handle_esm {
          return None;
        }
        // Skip if importMeta is disabled
        if matches!(
          parser.javascript_options.import_meta,
          Some(ImportMeta::None)
        ) {
          return None;
        }
        // Skip if node: false or node.filename is disabled
        if parser.compiler_options.node.is_none()
          || parser
            .compiler_options
            .node
            .as_ref()
            .is_some_and(|node_option| matches!(node_option.filename, NodeFilenameOption::False))
        {
          return None;
        }
      }
      DIRNAME => {
        // Skip CJS __dirname if not handling CJS
        if !self.handle_cjs {
          return None;
        }
        // Skip if node: false or node.dirname is disabled
        if parser.compiler_options.node.is_none()
          || parser
            .compiler_options
            .node
            .as_ref()
            .is_some_and(|node_option| matches!(node_option.dirname, NodeDirnameOption::False))
        {
          return None;
        }
      }
      expr_name::IMPORT_META_DIRNAME => {
        // Skip ESM import.meta.dirname if not handling ESM
        if !self.handle_esm {
          return None;
        }
        // Skip if importMeta is disabled
        if matches!(
          parser.javascript_options.import_meta,
          Some(ImportMeta::None)
        ) {
          return None;
        }
        // Skip if node: false or node.dirname is disabled
        if parser.compiler_options.node.is_none()
          || parser
            .compiler_options
            .node
            .as_ref()
            .is_some_and(|node_option| matches!(node_option.dirname, NodeDirnameOption::False))
        {
          return None;
        }
      }
      _ => return None,
    }

    parser.add_presentational_dependency(Box::new(ConstDependency::new(
      unary_expr.span().into(),
      "'string'".into(),
    )));
    Some(true)
  }

  fn evaluate_typeof<'a>(
    &self,
    parser: &mut JavascriptParser,
    expr: &'a swc_core::ecma::ast::UnaryExpr,
    for_name: &str,
  ) -> Option<eval::BasicEvaluatedExpression<'a>> {
    use crate::visitors::expr_name;

    match for_name {
      expr_name::IMPORT_META_FILENAME => {
        // Skip processing if importMeta is disabled
        if matches!(
          parser.javascript_options.import_meta,
          Some(ImportMeta::None)
        ) {
          return None;
        }
        // Skip processing if node: false or node.filename is disabled
        if parser.compiler_options.node.is_none()
          || parser
            .compiler_options
            .node
            .as_ref()
            .is_some_and(|node_option| matches!(node_option.filename, NodeFilenameOption::False))
        {
          return None;
        }
        Some(eval::evaluate_to_string(
          "string".to_string(),
          expr.span.real_lo(),
          expr.span.real_hi(),
        ))
      }
      expr_name::IMPORT_META_DIRNAME => {
        // Skip processing if importMeta is disabled
        if matches!(
          parser.javascript_options.import_meta,
          Some(ImportMeta::None)
        ) {
          return None;
        }
        // Skip processing if node: false or node.dirname is disabled
        if parser.compiler_options.node.is_none()
          || parser
            .compiler_options
            .node
            .as_ref()
            .is_some_and(|node_option| matches!(node_option.dirname, NodeDirnameOption::False))
        {
          return None;
        }
        Some(eval::evaluate_to_string(
          "string".to_string(),
          expr.span.real_lo(),
          expr.span.real_hi(),
        ))
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
  ) -> Option<crate::utils::eval::BasicEvaluatedExpression<'static>> {
    use crate::visitors::expr_name;

    if for_name == DIRNAME {
      // Skip CJS handling if not enabled
      if !self.handle_cjs {
        return None;
      }
      // Skip if node: false or node.dirname is disabled
      if parser.compiler_options.node.is_none()
        || parser
          .compiler_options
          .node
          .as_ref()
          .is_some_and(|node_option| matches!(node_option.dirname, NodeDirnameOption::False))
      {
        return None;
      }
      Some(eval::evaluate_to_string(
        get_context(parser.resource_data).as_str().to_string(),
        start,
        end,
      ))
    } else if for_name == FILENAME {
      // Skip CJS handling if not enabled
      if !self.handle_cjs {
        return None;
      }
      // Skip if node: false or node.filename is disabled
      if parser.compiler_options.node.is_none()
        || parser
          .compiler_options
          .node
          .as_ref()
          .is_some_and(|node_option| matches!(node_option.filename, NodeFilenameOption::False))
      {
        return None;
      }
      let resource = parse_resource(parser.resource_data.path()?.as_str())?;
      Some(eval::evaluate_to_string(
        resource.path.to_string(),
        start,
        end,
      ))
    } else if for_name == expr_name::IMPORT_META_FILENAME
      || for_name == expr_name::IMPORT_META_DIRNAME
    {
      // Skip ESM handling if not enabled
      if !self.handle_esm {
        return None;
      }
      // Skip processing if importMeta is disabled
      if matches!(
        parser.javascript_options.import_meta,
        Some(ImportMeta::None)
      ) {
        return None;
      }
      let property = if for_name == expr_name::IMPORT_META_FILENAME {
        NodeMetaProperty::Filename
      } else {
        NodeMetaProperty::Dirname
      };
      let value = Self::get_import_meta_eval_value(parser, property)?;
      Some(eval::evaluate_to_string(value, start, end))
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
    use crate::visitors::expr_name;

    // Skip ESM handling if not enabled
    if !self.handle_esm {
      return None;
    }

    let property = match for_name {
      expr_name::IMPORT_META_FILENAME => NodeMetaProperty::Filename,
      expr_name::IMPORT_META_DIRNAME => NodeMetaProperty::Dirname,
      _ => return None,
    };

    // Skip processing if importMeta is disabled
    if matches!(
      parser.javascript_options.import_meta,
      Some(ImportMeta::None)
    ) {
      return None;
    }

    let replacement = Self::get_import_meta_member_replacement(parser, property)?;
    parser.add_presentational_dependency(Box::new(ConstDependency::new(
      member_expr.span().into(),
      replacement.into(),
    )));
    Some(true)
  }

  fn import_meta_property_in_destructuring(
    &self,
    parser: &mut JavascriptParser,
    property: &DestructuringAssignmentProperty,
  ) -> Option<String> {
    // Skip ESM handling if not enabled
    if !self.handle_esm {
      return None;
    }

    // Skip processing if importMeta is disabled
    if matches!(
      parser.javascript_options.import_meta,
      Some(ImportMeta::None)
    ) {
      return None;
    }

    let meta_property = match property.id.as_str() {
      "filename" => NodeMetaProperty::Filename,
      "dirname" => NodeMetaProperty::Dirname,
      _ => return None,
    };

    let value = Self::get_import_meta_destructuring_value(parser, meta_property)?;
    Some(format!("{}: {value}", property.id))
  }
}
