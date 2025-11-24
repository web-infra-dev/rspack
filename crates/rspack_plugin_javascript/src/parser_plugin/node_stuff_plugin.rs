use rspack_core::{
  CachedConstDependency, ConstDependency, NodeDirnameOption, NodeFilenameOption, NodeGlobalOption,
  RuntimeGlobals, get_context, parse_resource,
};
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

const DIR_NAME: &str = "__dirname";
const FILE_NAME: &str = "__filename";
const GLOBAL: &str = "global";

pub struct NodeStuffPlugin;

impl JavascriptParserPlugin for NodeStuffPlugin {
  fn identifier(
    &self,
    parser: &mut JavascriptParser,
    ident: &swc_core::ecma::ast::Ident,
    for_name: &str,
  ) -> Option<bool> {
    let Some(node_option) = parser.compiler_options.node.as_ref() else {
      unreachable!("ensure only invoke `NodeStuffPlugin` when node options is enabled");
    };
    if for_name == DIR_NAME {
      let dirname = match node_option.dirname {
        NodeDirnameOption::Mock => Some("/".to_string()),
        NodeDirnameOption::WarnMock => Some("/".to_string()),
        NodeDirnameOption::NodeModule => {
          // `ExternalModuleDependency` extends `CachedConstDependency` in webpack.
          // We need to create two separate dependencies in Rspack.
          let external_url_dep = ExternalModuleDependency::new(
            "url".to_string(),
            vec![(
              "fileURLToPath".to_string(),
              "__webpack_fileURLToPath__".to_string(),
            )],
            None,
          );

          let external_path_dep = ExternalModuleDependency::new(
            "path".to_string(),
            vec![("dirname".to_string(), "__webpack_dirname__".to_string())],
            None,
          );

          let const_dep = CachedConstDependency::new(
            ident.span.into(),
            DIR_NAME.into(),
            "__webpack_dirname__(__webpack_fileURLToPath__(import.meta.url))"
              .to_string()
              .into(),
          );

          parser.add_presentational_dependency(Box::new(external_url_dep));
          parser.add_presentational_dependency(Box::new(external_path_dep));
          parser.add_presentational_dependency(Box::new(const_dep));
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
        _ => None,
      };
      if let Some(dirname) = dirname {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          ident.span.into(),
          serde_json::to_string(&dirname)
            .expect("should render dirname")
            .into(),
          None,
        )));
        return Some(true);
      }
    } else if for_name == FILE_NAME {
      let filename = match node_option.filename {
        NodeFilenameOption::Mock => Some("/index.js".to_string()),
        NodeFilenameOption::WarnMock => Some("/index.js".to_string()),
        NodeFilenameOption::NodeModule => {
          // `ExternalModuleDependency` extends `CachedConstDependency` in webpack.
          // We need to create two separate dependencies in Rspack.
          let external_dep = ExternalModuleDependency::new(
            "url".to_string(),
            vec![(
              "fileURLToPath".to_string(),
              "__webpack_fileURLToPath__".to_string(),
            )],
            None,
          );

          let const_dep = CachedConstDependency::new(
            ident.span.into(),
            FILE_NAME.into(),
            "__webpack_fileURLToPath__(import.meta.url)"
              .to_string()
              .into(),
          );

          parser.add_presentational_dependency(Box::new(external_dep));
          parser.add_presentational_dependency(Box::new(const_dep));
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
        _ => None,
      };
      if let Some(filename) = filename {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          ident.span.into(),
          serde_json::to_string(&filename)
            .expect("should render filename")
            .into(),
          None,
        )));
        return Some(true);
      }
    } else if for_name == GLOBAL
      && matches!(
        node_option.global,
        NodeGlobalOption::True | NodeGlobalOption::Warn
      )
    {
      parser.add_presentational_dependency(Box::new(ConstDependency::new(
        ident.span.into(),
        RuntimeGlobals::GLOBAL.name().into(),
        Some(RuntimeGlobals::GLOBAL),
      )));
      return Some(true);
    }
    None
  }

  fn rename(&self, parser: &mut JavascriptParser, expr: &Expr, for_name: &str) -> Option<bool> {
    let Some(node_option) = parser.compiler_options.node.as_ref() else {
      unreachable!("ensure only invoke `NodeStuffPlugin` when node options is enabled");
    };
    if for_name == GLOBAL
      && matches!(
        node_option.global,
        NodeGlobalOption::True | NodeGlobalOption::Warn
      )
    {
      parser.add_presentational_dependency(Box::new(ConstDependency::new(
        expr.span().into(),
        RuntimeGlobals::GLOBAL.name().into(),
        Some(RuntimeGlobals::GLOBAL),
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
      FILE_NAME | expr_name::IMPORT_META_FILENAME => {
        // Skip processing if importMeta is disabled
        if parser.javascript_options.import_meta == Some(false) {
          return None;
        }
        // Skip processing if node.filename is disabled
        if parser
          .compiler_options
          .node
          .as_ref()
          .is_some_and(|node_option| matches!(node_option.filename, NodeFilenameOption::False))
        {
          return None;
        }
        // fallthrough
      }
      DIR_NAME | expr_name::IMPORT_META_DIRNAME => {
        // Skip processing if importMeta is disabled
        if parser.javascript_options.import_meta == Some(false) {
          return None;
        }
        // Skip processing if node.dirname is disabled
        if parser
          .compiler_options
          .node
          .as_ref()
          .is_some_and(|node_option| matches!(node_option.dirname, NodeDirnameOption::False))
        {
          return None;
        }
        // fallthrough
      }
      _ => return None,
    }

    parser.add_presentational_dependency(Box::new(ConstDependency::new(
      unary_expr.span().into(),
      "'string'".into(),
      None,
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
        if parser.javascript_options.import_meta == Some(false) {
          return None;
        }
        // Skip processing if node.filename is disabled
        if parser
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
        if parser.javascript_options.import_meta == Some(false) {
          return None;
        }
        // Skip processing if node.dirname is disabled
        if parser
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

    if for_name == DIR_NAME {
      if parser
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
    } else if for_name == FILE_NAME {
      if parser
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
    } else if for_name == expr_name::IMPORT_META_FILENAME {
      // Skip processing if importMeta is disabled
      if parser.javascript_options.import_meta == Some(false) {
        return None;
      }
      // Skip processing if node.filename is disabled
      if parser
        .compiler_options
        .node
        .as_ref()
        .is_some_and(|node_option| matches!(node_option.filename, NodeFilenameOption::False))
      {
        return None;
      }
      let filename = Url::from_file_path(parser.resource_data.resource())
        .expect("should be a path")
        .to_file_path()
        .expect("should be a path")
        .to_string_lossy()
        .into_owned();
      Some(eval::evaluate_to_string(filename, start, end))
    } else if for_name == expr_name::IMPORT_META_DIRNAME {
      // Skip processing if importMeta is disabled
      if parser.javascript_options.import_meta == Some(false) {
        return None;
      }
      // Skip processing if node.dirname is disabled
      if parser
        .compiler_options
        .node
        .as_ref()
        .is_some_and(|node_option| matches!(node_option.dirname, NodeDirnameOption::False))
      {
        return None;
      }
      let dirname = Url::from_file_path(parser.resource_data.resource())
        .expect("should be a path")
        .to_file_path()
        .expect("should be a path")
        .parent()
        .expect("should have a parent")
        .to_string_lossy()
        .into_owned();
      Some(eval::evaluate_to_string(dirname, start, end))
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

    match for_name {
      expr_name::IMPORT_META_FILENAME => {
        // Skip processing if importMeta is disabled
        if parser.javascript_options.import_meta == Some(false) {
          return None;
        }
        // Skip processing if node.filename is disabled
        if parser
          .compiler_options
          .node
          .as_ref()
          .is_some_and(|node_option| matches!(node_option.filename, NodeFilenameOption::False))
        {
          return None;
        }
        // import.meta.filename
        let filename = Url::from_file_path(parser.resource_data.resource())
          .expect("should be a path")
          .to_file_path()
          .expect("should be a path")
          .to_string_lossy()
          .into_owned();
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          member_expr.span().into(),
          format!("'{filename}'").into(),
          None,
        )));
        Some(true)
      }
      expr_name::IMPORT_META_DIRNAME => {
        // Skip processing if importMeta is disabled
        if parser.javascript_options.import_meta == Some(false) {
          return None;
        }
        // Skip processing if node.dirname is disabled
        if parser
          .compiler_options
          .node
          .as_ref()
          .is_some_and(|node_option| matches!(node_option.dirname, NodeDirnameOption::False))
        {
          return None;
        }
        // import.meta.dirname
        let dirname = Url::from_file_path(parser.resource_data.resource())
          .expect("should be a path")
          .to_file_path()
          .expect("should be a path")
          .parent()
          .expect("should have a parent")
          .to_string_lossy()
          .into_owned();
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          member_expr.span().into(),
          format!("'{dirname}'").into(),
          None,
        )));
        Some(true)
      }
      _ => None,
    }
  }

  fn import_meta_property_in_destructuring(
    &self,
    parser: &mut JavascriptParser,
    property: &DestructuringAssignmentProperty,
  ) -> Option<String> {
    // Skip processing if importMeta is disabled
    if parser.javascript_options.import_meta == Some(false) {
      return None;
    }
    match property.id.as_str() {
      "filename" => {
        // Skip processing if node.filename is disabled
        if parser
          .compiler_options
          .node
          .as_ref()
          .is_some_and(|node_option| matches!(node_option.filename, NodeFilenameOption::False))
        {
          return None;
        }
        // This is the same as the url.fileURLToPath() of the import.meta.url
        let filename = Url::from_file_path(parser.resource_data.resource())
          .expect("should be a path")
          .to_file_path()
          .expect("should be a path")
          .to_string_lossy()
          .into_owned();
        Some(format!(r#"filename: "{}""#, filename))
      }
      "dirname" => {
        // Skip processing if node.dirname is disabled
        if parser
          .compiler_options
          .node
          .as_ref()
          .is_some_and(|node_option| matches!(node_option.dirname, NodeDirnameOption::False))
        {
          return None;
        }
        // This is the same as the path.dirname() of the import.meta.filename
        let dirname = Url::from_file_path(parser.resource_data.resource())
          .expect("should be a path")
          .to_file_path()
          .expect("should be a path")
          .parent()
          .expect("should have a parent")
          .to_string_lossy()
          .into_owned();
        Some(format!(r#"dirname: "{}""#, dirname))
      }
      _ => None,
    }
  }
}
