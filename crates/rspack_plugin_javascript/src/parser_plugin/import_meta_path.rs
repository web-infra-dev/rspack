use concat_string::concat_string;
use rspack_core::{
  CachedConstDependency, CachedConstDependencyPlace, DependencyRange, ImportMetaKnownProperties,
  NodeDirnameOption, NodeFilenameOption, NodeOption, parse_resource,
};
use rspack_error::{Diagnostic, cyan, yellow};
use sugar_path::SugarPath;
use url::Url;

use crate::{dependency::ExternalModuleDependency, visitors::JavascriptParser};

const MOCK_DIRNAME: &str = "/";
const MOCK_FILENAME: &str = "/index.js";

fn mock_value(property: ImportMetaKnownProperties) -> &'static str {
  match property {
    ImportMetaKnownProperties::FILENAME => MOCK_FILENAME,
    ImportMetaKnownProperties::DIRNAME => MOCK_DIRNAME,
    _ => unreachable!("only import.meta.dirname and import.meta.filename are supported"),
  }
}

fn import_meta_name(property: ImportMetaKnownProperties) -> &'static str {
  match property {
    ImportMetaKnownProperties::FILENAME => "import.meta.filename",
    ImportMetaKnownProperties::DIRNAME => "import.meta.dirname",
    _ => unreachable!("only import.meta.dirname and import.meta.filename are supported"),
  }
}

fn cjs_name(property: ImportMetaKnownProperties) -> &'static str {
  match property {
    ImportMetaKnownProperties::FILENAME => "__filename",
    ImportMetaKnownProperties::DIRNAME => "__dirname",
    _ => unreachable!("only import.meta.dirname and import.meta.filename are supported"),
  }
}

fn warning_code(property: ImportMetaKnownProperties) -> &'static str {
  match property {
    ImportMetaKnownProperties::FILENAME => "NODE_IMPORT_META_FILENAME",
    ImportMetaKnownProperties::DIRNAME => "NODE_IMPORT_META_DIRNAME",
    _ => unreachable!("only import.meta.dirname and import.meta.filename are supported"),
  }
}

fn warning_message(property: ImportMetaKnownProperties) -> String {
  match property {
    ImportMetaKnownProperties::FILENAME => concat_string!(
      "\"",
      yellow(&"import.meta.filename").to_string(),
      "\" is used and has been mocked. ",
      "Remove it from your code, or set `",
      cyan(&"node.__filename").to_string(),
      "` to disable this warning."
    ),
    ImportMetaKnownProperties::DIRNAME => concat_string!(
      "\"",
      yellow(&"import.meta.dirname").to_string(),
      "\" is used and has been mocked. ",
      "Remove it from your code, or set `",
      cyan(&"node.__dirname").to_string(),
      "` to disable this warning."
    ),
    _ => unreachable!("only import.meta.dirname and import.meta.filename are supported"),
  }
}

fn node_module_runtime_expr(property: ImportMetaKnownProperties) -> &'static str {
  match property {
    ImportMetaKnownProperties::FILENAME => "__rspack_fileURLToPath(import.meta.url)",
    ImportMetaKnownProperties::DIRNAME => {
      "__rspack_dirname(__rspack_fileURLToPath(import.meta.url))"
    }
    _ => unreachable!("only import.meta.dirname and import.meta.filename are supported"),
  }
}

fn import_meta_cached_identifier(property: ImportMetaKnownProperties) -> &'static str {
  match property {
    ImportMetaKnownProperties::FILENAME => "__rspack_import_meta_filename__",
    ImportMetaKnownProperties::DIRNAME => "__rspack_import_meta_dirname__",
    _ => unreachable!("only import.meta.dirname and import.meta.filename are supported"),
  }
}

fn is_disabled(property: ImportMetaKnownProperties, node_option: &NodeOption) -> bool {
  match property {
    ImportMetaKnownProperties::FILENAME => {
      matches!(node_option.filename, NodeFilenameOption::False)
    }
    ImportMetaKnownProperties::DIRNAME => matches!(node_option.dirname, NodeDirnameOption::False),
    _ => unreachable!("only import.meta.dirname and import.meta.filename are supported"),
  }
}

fn is_mock(property: ImportMetaKnownProperties, node_option: &NodeOption) -> bool {
  match property {
    ImportMetaKnownProperties::FILENAME => matches!(node_option.filename, NodeFilenameOption::Mock),
    ImportMetaKnownProperties::DIRNAME => matches!(node_option.dirname, NodeDirnameOption::Mock),
    _ => unreachable!("only import.meta.dirname and import.meta.filename are supported"),
  }
}

fn is_warn_mock(property: ImportMetaKnownProperties, node_option: &NodeOption) -> bool {
  match property {
    ImportMetaKnownProperties::FILENAME => {
      matches!(node_option.filename, NodeFilenameOption::WarnMock)
    }
    ImportMetaKnownProperties::DIRNAME => {
      matches!(node_option.dirname, NodeDirnameOption::WarnMock)
    }
    _ => unreachable!("only import.meta.dirname and import.meta.filename are supported"),
  }
}

fn is_true(property: ImportMetaKnownProperties, node_option: &NodeOption) -> bool {
  match property {
    ImportMetaKnownProperties::FILENAME => matches!(node_option.filename, NodeFilenameOption::True),
    ImportMetaKnownProperties::DIRNAME => matches!(node_option.dirname, NodeDirnameOption::True),
    _ => unreachable!("only import.meta.dirname and import.meta.filename are supported"),
  }
}

fn is_eval_only(property: ImportMetaKnownProperties, node_option: &NodeOption) -> bool {
  match property {
    ImportMetaKnownProperties::FILENAME => {
      matches!(node_option.filename, NodeFilenameOption::EvalOnly)
    }
    ImportMetaKnownProperties::DIRNAME => {
      matches!(node_option.dirname, NodeDirnameOption::EvalOnly)
    }
    _ => unreachable!("only import.meta.dirname and import.meta.filename are supported"),
  }
}

fn is_node_module(property: ImportMetaKnownProperties, node_option: &NodeOption) -> bool {
  match property {
    ImportMetaKnownProperties::FILENAME => {
      matches!(node_option.filename, NodeFilenameOption::NodeModule)
    }
    ImportMetaKnownProperties::DIRNAME => {
      matches!(node_option.dirname, NodeDirnameOption::NodeModule)
    }
    _ => unreachable!("only import.meta.dirname and import.meta.filename are supported"),
  }
}

pub(crate) fn should_handle_import_meta_path(
  parser: &JavascriptParser,
  property: ImportMetaKnownProperties,
) -> bool {
  parser
    .compiler_options
    .node
    .as_ref()
    .is_some_and(|node_option| !is_disabled(property, node_option))
}

fn get_relative_path(
  parser: &JavascriptParser,
  property: ImportMetaKnownProperties,
) -> Option<String> {
  match property {
    ImportMetaKnownProperties::FILENAME => Some(
      parser
        .resource_data
        .path()?
        .as_std_path()
        .relative(&parser.compiler_options.context)
        .to_string_lossy()
        .to_string(),
    ),
    ImportMetaKnownProperties::DIRNAME => Some(
      parser
        .resource_data
        .path()?
        .parent()?
        .as_std_path()
        .relative(&parser.compiler_options.context)
        .to_string_lossy()
        .to_string(),
    ),
    _ => unreachable!("only import.meta.dirname and import.meta.filename are supported"),
  }
}

fn get_absolute_path(
  parser: &JavascriptParser,
  property: ImportMetaKnownProperties,
) -> Option<String> {
  let path = Url::from_file_path(parser.resource_data.resource())
    .expect("should be a path")
    .to_file_path()
    .expect("should be a path");

  match property {
    ImportMetaKnownProperties::FILENAME => Some(path.to_string_lossy().into_owned()),
    ImportMetaKnownProperties::DIRNAME => Some(
      path
        .parent()
        .expect("should have a parent")
        .to_string_lossy()
        .into_owned(),
    ),
    _ => unreachable!("only import.meta.dirname and import.meta.filename are supported"),
  }
}

fn get_eval_only_path(
  parser: &JavascriptParser,
  property: ImportMetaKnownProperties,
) -> Option<String> {
  match property {
    ImportMetaKnownProperties::FILENAME => {
      let resource = parse_resource(parser.resource_data.path()?.as_str())?;
      Some(resource.path.to_string())
    }
    ImportMetaKnownProperties::DIRNAME => Some(parser.resource_data.path()?.parent()?.to_string()),
    _ => unreachable!("only import.meta.dirname and import.meta.filename are supported"),
  }
}

fn add_node_module_dependencies(
  parser: &mut JavascriptParser,
  property: ImportMetaKnownProperties,
) {
  let external_url_dep = ExternalModuleDependency::new(
    "url".to_string(),
    vec![(
      "fileURLToPath".to_string(),
      "__rspack_fileURLToPath".to_string(),
    )],
    None,
  );
  parser.add_presentational_dependency(Box::new(external_url_dep));

  if property == ImportMetaKnownProperties::DIRNAME {
    let external_path_dep = ExternalModuleDependency::new(
      "path".to_string(),
      vec![("dirname".to_string(), "__rspack_dirname".to_string())],
      None,
    );
    parser.add_presentational_dependency(Box::new(external_path_dep));
  }
}

fn add_import_meta_cached_dependency(
  parser: &mut JavascriptParser,
  range: Option<DependencyRange>,
  property: ImportMetaKnownProperties,
  content: impl Into<Box<str>>,
) -> String {
  let identifier = import_meta_cached_identifier(property);
  let const_dep = match range {
    Some(range) => CachedConstDependency::new_with_place(
      range,
      identifier.into(),
      content.into(),
      CachedConstDependencyPlace::Chunk,
    ),
    None => CachedConstDependency::new_without_replacement(
      identifier.into(),
      content.into(),
      CachedConstDependencyPlace::Chunk,
    ),
  };
  parser.add_presentational_dependency(Box::new(const_dep));
  identifier.to_string()
}

pub(crate) fn get_import_meta_eval_value(
  parser: &JavascriptParser,
  property: ImportMetaKnownProperties,
) -> Option<String> {
  let node_option = parser.compiler_options.node.as_ref()?;

  if is_disabled(property, node_option) {
    return None;
  }

  if is_mock(property, node_option) || is_warn_mock(property, node_option) {
    return Some(mock_value(property).to_string());
  }

  if is_true(property, node_option) {
    return get_relative_path(parser, property);
  }

  if is_eval_only(property, node_option) {
    return get_eval_only_path(parser, property);
  }

  if is_node_module(property, node_option) {
    return get_absolute_path(parser, property);
  }

  None
}

pub(crate) fn get_import_meta_member_replacement(
  parser: &mut JavascriptParser,
  property: ImportMetaKnownProperties,
) -> Option<String> {
  let node_option = match parser.compiler_options.node.as_ref() {
    None => return Some(import_meta_name(property).to_string()),
    Some(opt) => opt,
  };

  if is_disabled(property, node_option) {
    return Some(import_meta_name(property).to_string());
  }

  if is_mock(property, node_option) {
    return Some(rspack_util::json_stringify_str(mock_value(property)));
  }

  if is_warn_mock(property, node_option) {
    parser.add_warning(Diagnostic::warn(
      warning_code(property).to_string(),
      warning_message(property),
    ));
    return Some(rspack_util::json_stringify_str(mock_value(property)));
  }

  if is_true(property, node_option) {
    let path = get_relative_path(parser, property)?;
    return Some(rspack_util::json_stringify_str(&path));
  }

  if is_eval_only(property, node_option) {
    return Some(if parser.compiler_options.output.module {
      add_import_meta_cached_dependency(parser, None, property, import_meta_name(property))
    } else {
      cjs_name(property).to_string()
    });
  }

  if is_node_module(property, node_option) {
    if parser.compiler_options.output.module
      && parser
        .compiler_options
        .output
        .environment
        .supports_import_meta_dirname_and_filename()
    {
      return Some(add_import_meta_cached_dependency(
        parser,
        None,
        property,
        import_meta_name(property),
      ));
    }
    add_node_module_dependencies(parser, property);
    return Some(add_import_meta_cached_dependency(
      parser,
      None,
      property,
      node_module_runtime_expr(property),
    ));
  }

  None
}
