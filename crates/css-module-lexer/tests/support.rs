pub(crate) use css_module_lexer::{
  Dependency, DependencyContext, ImportAttributes, Lexer, Mode, ModeData, Range, UrlRangeKind,
  Warning, WarningKind, collect_dependencies, lex_dependencies,
};
pub(crate) use indoc::indoc;
pub(crate) use smallvec::SmallVec;

pub(crate) fn assert_warning(input: &str, warning: &Warning, range_content: &str) {
  assert_eq!(
    Lexer::slice_range(input, warning.range()).expect("test setup must produce the expected value"),
    range_content
  );
}

pub(crate) fn assert_url_dependency(
  input: &str,
  dependency: &Dependency,
  request: &str,
  kind: UrlRangeKind,
  range_content: &str,
) {
  let Dependency::Url {
    request: req,
    range,
    kind: k,
  } = dependency
  else {
    panic!("unexpected dependency");
  };
  assert_eq!(*req, request);
  assert_eq!(*k, kind);
  assert_eq!(
    Lexer::slice_range(input, range).expect("test setup must produce the expected value"),
    range_content
  );
}

pub(crate) type ResolvedImport<'context, 's> =
  (&'s str, &'context Range, &'context ImportAttributes<'s>);

pub(crate) fn assert_import_dependency(
  input: &str,
  (actual_request, range, attributes): ResolvedImport<'_, '_>,
  request: &str,
  layer: Option<&str>,
  supports: Option<&str>,
  media: Option<&str>,
  range_content: &str,
) {
  assert_eq!(actual_request, request);
  assert_eq!(attributes.layer(), layer);
  assert_eq!(attributes.supports(), supports);
  assert_eq!(attributes.media(), media);
  assert_eq!(
    Lexer::slice_range(input, range).expect("test setup must produce the expected value"),
    range_content
  );
}

pub(crate) fn import_dependency<'context, 's>(
  context: &'context DependencyContext<'s>,
  dependency_index: usize,
) -> ResolvedImport<'context, 's> {
  let Dependency::Import {
    request,
    range,
    attributes,
  } = &context[dependency_index]
  else {
    panic!("unexpected dependency");
  };
  (*request, range, context.import_attributes(*attributes))
}

pub(crate) fn assert_icss_import_url_dependency(
  input: &str,
  dependency: &Dependency,
  name: &str,
  range_content: &str,
  name_range_content: &str,
) {
  let Dependency::ICSSImportUrl {
    name: actual_name,
    range,
    name_range,
  } = dependency
  else {
    panic!("unexpected dependency");
  };
  assert_eq!(*actual_name, name);
  assert_eq!(
    Lexer::slice_range(input, range).expect("test setup must produce the expected value"),
    range_content
  );
  assert_eq!(
    Lexer::slice_range(input, name_range).expect("test setup must produce the expected value"),
    name_range_content
  );
}

pub(crate) fn assert_local_class_dependency(
  input: &str,
  dependency: &Dependency,
  name: &str,
  explicit: bool,
) {
  let Dependency::LocalClass {
    name: actual_name,
    explicit: actual_explicit,
    range,
  } = dependency
  else {
    panic!("unexpected dependency");
  };
  assert_eq!(*actual_name, name);
  assert_eq!(*actual_explicit, explicit);
  assert_eq!(
    Lexer::slice_range(input, range).expect("test setup must produce the expected value"),
    name
  );
}

pub(crate) fn assert_local_id_dependency(
  input: &str,
  dependency: &Dependency,
  name: &str,
  explicit: bool,
) {
  let Dependency::LocalId {
    name: actual_name,
    explicit: actual_explicit,
    range,
  } = dependency
  else {
    panic!("unexpected dependency");
  };
  assert_eq!(*actual_name, name);
  assert_eq!(*actual_explicit, explicit);
  assert_eq!(
    Lexer::slice_range(input, range).expect("test setup must produce the expected value"),
    name
  );
}

pub(crate) fn assert_local_var_dependency(
  input: &str,
  dependency: &Dependency,
  name: &str,
  from: Option<&str>,
) {
  let Dependency::LocalVar {
    name: actual_name,
    range,
    from: actual_from,
    from_is_global,
  } = dependency
  else {
    panic!("unexpected dependency");
  };
  assert_eq!(*actual_name, name);
  assert_eq!(*actual_from, from);
  assert_eq!(*from_is_global, from == Some("global"));
  assert_eq!(
    Lexer::slice_range(input, range).expect("test setup must produce the expected value"),
    format!("--{name}")
  );
}

pub(crate) fn assert_local_var_decl_dependency(input: &str, dependency: &Dependency, name: &str) {
  let Dependency::LocalVarDecl {
    range,
    name: actual_name,
  } = dependency
  else {
    panic!("unexpected dependency");
  };
  assert_eq!(*actual_name, name);
  assert_eq!(
    Lexer::slice_range(input, range).expect("test setup must produce the expected value"),
    format!("--{name}")
  );
}

pub(crate) fn assert_local_property_decl_dependency(
  input: &str,
  dependency: &Dependency,
  name: &str,
) {
  let Dependency::LocalPropertyDecl {
    name: actual_name,
    range,
  } = dependency
  else {
    panic!("unexpected dependency");
  };
  assert_eq!(*actual_name, name);
  assert_eq!(
    Lexer::slice_range(input, range).expect("test setup must produce the expected value"),
    format!("--{name}")
  );
}

pub(crate) fn assert_local_keyframes_decl_dependency(
  input: &str,
  dependency: &Dependency,
  name: &str,
) {
  let Dependency::LocalKeyframesDecl {
    name: actual_name,
    range,
  } = dependency
  else {
    panic!("unexpected dependency");
  };
  assert_eq!(*actual_name, name);
  assert_eq!(
    Lexer::slice_range(input, range).expect("test setup must produce the expected value"),
    name
  );
}

pub(crate) fn assert_local_keyframes_dependency(input: &str, dependency: &Dependency, name: &str) {
  let Dependency::LocalKeyframes {
    name: actual_name,
    range,
  } = dependency
  else {
    panic!("unexpected dependency");
  };
  assert_eq!(*actual_name, name);
  assert_eq!(
    Lexer::slice_range(input, range).expect("test setup must produce the expected value"),
    name
  );
}

pub(crate) fn assert_local_counter_style_decl_dependency(
  input: &str,
  dependency: &Dependency,
  name: &str,
) {
  let Dependency::LocalCounterStyleDecl {
    name: actual_name,
    range,
  } = dependency
  else {
    panic!("unexpected dependency");
  };
  assert_eq!(*actual_name, name);
  assert_eq!(
    Lexer::slice_range(input, range).expect("test setup must produce the expected value"),
    name
  );
}

pub(crate) fn assert_local_counter_style_dependency(
  input: &str,
  dependency: &Dependency,
  name: &str,
) {
  let Dependency::LocalCounterStyle {
    name: actual_name,
    range,
  } = dependency
  else {
    panic!("unexpected dependency");
  };
  assert_eq!(*actual_name, name);
  assert_eq!(
    Lexer::slice_range(input, range).expect("test setup must produce the expected value"),
    name
  );
}

pub(crate) fn assert_local_font_palette_decl_dependency(
  input: &str,
  dependency: &Dependency,
  name: &str,
) {
  let Dependency::LocalFontPaletteDecl {
    name: actual_name,
    range,
  } = dependency
  else {
    panic!("unexpected dependency");
  };
  assert_eq!(*actual_name, name);
  assert_eq!(
    Lexer::slice_range(input, range).expect("test setup must produce the expected value"),
    format!("--{name}")
  );
}

pub(crate) fn assert_local_font_palette_dependency(
  input: &str,
  dependency: &Dependency,
  name: &str,
) {
  let Dependency::LocalFontPalette {
    name: actual_name,
    range,
  } = dependency
  else {
    panic!("unexpected dependency");
  };
  assert_eq!(*actual_name, name);
  assert_eq!(
    Lexer::slice_range(input, range).expect("test setup must produce the expected value"),
    format!("--{name}")
  );
}

pub(crate) fn assert_local_container_dependency(input: &str, dependency: &Dependency, name: &str) {
  let Dependency::LocalContainer {
    name: actual_name,
    range,
  } = dependency
  else {
    panic!("unexpected dependency");
  };
  assert_eq!(*actual_name, name);
  assert_eq!(
    Lexer::slice_range(input, range).expect("test setup must produce the expected value"),
    name
  );
}

pub(crate) fn assert_local_container_decl_dependency(
  input: &str,
  dependency: &Dependency,
  name: &str,
) {
  let Dependency::LocalContainerDecl {
    name: actual_name,
    range,
  } = dependency
  else {
    panic!("unexpected dependency");
  };
  assert_eq!(*actual_name, name);
  assert_eq!(
    Lexer::slice_range(input, range).expect("test setup must produce the expected value"),
    name
  );
}

pub(crate) fn assert_local_function_dependency(input: &str, dependency: &Dependency, name: &str) {
  let Dependency::LocalFunction {
    name: actual_name,
    range,
  } = dependency
  else {
    panic!("unexpected dependency");
  };
  assert_eq!(*actual_name, name);
  assert_eq!(
    Lexer::slice_range(input, range).expect("test setup must produce the expected value"),
    format!("--{name}")
  );
}

pub(crate) fn assert_local_function_decl_dependency(
  input: &str,
  dependency: &Dependency,
  name: &str,
) {
  let Dependency::LocalFunctionDecl {
    name: actual_name,
    range,
  } = dependency
  else {
    panic!("unexpected dependency");
  };
  assert_eq!(*actual_name, name);
  assert_eq!(
    Lexer::slice_range(input, range).expect("test setup must produce the expected value"),
    format!("--{name}")
  );
}

pub(crate) fn assert_local_grid_dependency(input: &str, dependency: &Dependency, name: &str) {
  let Dependency::LocalGrid {
    name: actual_name,
    range,
  } = dependency
  else {
    panic!("unexpected dependency");
  };
  assert_eq!(*actual_name, name);
  assert_eq!(
    Lexer::slice_range(input, range).expect("test setup must produce the expected value"),
    name
  );
}

pub(crate) fn assert_local_grid_decl_dependency(input: &str, dependency: &Dependency, name: &str) {
  let Dependency::LocalGridDecl {
    name: actual_name,
    range,
  } = dependency
  else {
    panic!("unexpected dependency");
  };
  assert_eq!(*actual_name, name);
  assert_eq!(
    Lexer::slice_range(input, range).expect("test setup must produce the expected value"),
    name
  );
}

pub(crate) fn assert_composes_dependency(
  input: &str,
  context: &DependencyContext,
  dependency: &Dependency,
  local_classes: &str,
  names: &str,
  from: Option<&str>,
  range_content: &str,
) {
  let Dependency::Composes {
    local_classes: actual_local_classes,
    names: actual_names,
    from: actual_from,
    range,
    ..
  } = dependency
  else {
    panic!("unexpected dependency");
  };
  assert_eq!(
    context
      .composes_local_classes(*actual_local_classes)
      .iter()
      .copied()
      .collect::<SmallVec<[&str; 2]>>(),
    local_classes.split(' ').collect::<SmallVec<[&str; 2]>>()
  );
  assert_eq!(
    context
      .composes_names(*actual_names)
      .iter()
      .copied()
      .collect::<SmallVec<[&str; 2]>>(),
    names.split(' ').collect::<SmallVec<[&str; 2]>>()
  );
  assert_eq!(*actual_from, from);
  assert_eq!(
    Lexer::slice_range(input, range).expect("test setup must produce the expected value"),
    range_content
  );
}

pub(crate) fn assert_replace_dependency(
  input: &str,
  dependency: &Dependency,
  content: &str,
  range_content: &str,
) {
  let Dependency::Replace {
    content: actual_content,
    range,
  } = dependency
  else {
    panic!("unexpected dependency");
  };
  assert_eq!(*actual_content, content);
  assert_eq!(
    Lexer::slice_range(input, range).expect("test setup must produce the expected value"),
    range_content
  );
}

pub(crate) fn assert_charset_dependency(
  input: &str,
  dependency: &Dependency,
  value: &str,
  range_content: &str,
) {
  let Dependency::Charset {
    value: actual_value,
    range,
  } = dependency
  else {
    panic!("unexpected dependency");
  };
  assert_eq!(*actual_value, value);
  assert_eq!(
    Lexer::slice_range(input, range).expect("test setup must produce the expected value"),
    range_content
  );
}

pub(crate) fn assert_icss_import_from_dependency(
  _input: &str,
  dependency: &Dependency,
  path: &str,
) {
  let Dependency::ICSSImportFrom { path: actual_path } = dependency else {
    panic!("unexpected dependency");
  };
  assert_eq!(*actual_path, path);
}

pub(crate) fn assert_icss_import_value_dependency(
  _input: &str,
  dependency: &Dependency,
  prop: &str,
  value: &str,
) {
  let Dependency::ICSSImportValue {
    prop: actual_prop,
    value: actual_value,
  } = dependency
  else {
    panic!("unexpected dependency");
  };
  assert_eq!(*actual_prop, prop);
  assert_eq!(*actual_value, value);
}

pub(crate) fn assert_icss_export_value_dependency(
  _input: &str,
  dependency: &Dependency,
  prop: &str,
  value: &str,
) {
  let Dependency::ICSSExportValue {
    prop: actual_prop,
    value: actual_value,
  } = dependency
  else {
    panic!("unexpected dependency");
  };
  assert_eq!(*actual_prop, prop);
  assert_eq!(*actual_value, value);
}

pub(crate) fn assert_icss_symbol_dependency(
  input: &str,
  dependency: &Dependency,
  name: &str,
  range_content: &str,
) {
  let Dependency::ICSSSymbol {
    name: actual_name,
    range,
  } = dependency
  else {
    panic!("unexpected dependency");
  };
  assert_eq!(*actual_name, name);
  assert_eq!(
    Lexer::slice_range(input, range).expect("test setup must produce the expected value"),
    range_content
  );
}
