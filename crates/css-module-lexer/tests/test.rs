mod postcss_modules;

use css_module_lexer::{
  Dependency, DependencyContext, ImportAttributes, Lexer, Mode, ModeData, Range, UrlRangeKind,
  Warning, WarningKind, collect_dependencies, lex_dependencies,
};
use indoc::indoc;
use smallvec::SmallVec;

fn assert_warning(input: &str, warning: &Warning, range_content: &str) {
  assert_eq!(
    Lexer::slice_range(input, warning.range()).unwrap(),
    range_content
  );
}

fn assert_url_dependency(
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
  assert_eq!(Lexer::slice_range(input, range).unwrap(), range_content);
}

type ResolvedImport<'context, 's> = (&'s str, &'context Range, &'context ImportAttributes<'s>);

fn assert_import_dependency(
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
  assert_eq!(Lexer::slice_range(input, range).unwrap(), range_content);
}

fn import_dependency<'context, 's>(
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

fn assert_icss_import_url_dependency(
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
  assert_eq!(Lexer::slice_range(input, range).unwrap(), range_content);
  assert_eq!(
    Lexer::slice_range(input, name_range).unwrap(),
    name_range_content
  );
}

fn assert_local_class_dependency(input: &str, dependency: &Dependency, name: &str, explicit: bool) {
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
  assert_eq!(Lexer::slice_range(input, range).unwrap(), name);
}

fn assert_local_id_dependency(input: &str, dependency: &Dependency, name: &str, explicit: bool) {
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
  assert_eq!(Lexer::slice_range(input, range).unwrap(), name);
}

fn assert_local_var_dependency(
  input: &str,
  dependency: &Dependency,
  name: &str,
  from: Option<&str>,
) {
  let Dependency::LocalVar {
    name: actual_name,
    range,
    from: actual_from,
  } = dependency
  else {
    panic!("unexpected dependency");
  };
  assert_eq!(*actual_name, name);
  assert_eq!(*actual_from, from);
  assert_eq!(
    Lexer::slice_range(input, range).unwrap(),
    format!("--{}", name)
  );
}

fn assert_local_var_decl_dependency(input: &str, dependency: &Dependency, name: &str) {
  let Dependency::LocalVarDecl {
    range,
    name: actual_name,
  } = dependency
  else {
    panic!("unexpected dependency");
  };
  assert_eq!(*actual_name, name);
  assert_eq!(
    Lexer::slice_range(input, range).unwrap(),
    format!("--{}", name)
  );
}

fn assert_local_property_decl_dependency(input: &str, dependency: &Dependency, name: &str) {
  let Dependency::LocalPropertyDecl {
    name: actual_name,
    range,
  } = dependency
  else {
    panic!("unexpected dependency");
  };
  assert_eq!(*actual_name, name);
  assert_eq!(
    Lexer::slice_range(input, range).unwrap(),
    format!("--{}", name)
  );
}

fn assert_local_keyframes_decl_dependency(input: &str, dependency: &Dependency, name: &str) {
  let Dependency::LocalKeyframesDecl {
    name: actual_name,
    range,
  } = dependency
  else {
    panic!("unexpected dependency");
  };
  assert_eq!(*actual_name, name);
  assert_eq!(Lexer::slice_range(input, range).unwrap(), name);
}

fn assert_local_keyframes_dependency(input: &str, dependency: &Dependency, name: &str) {
  let Dependency::LocalKeyframes {
    name: actual_name,
    range,
  } = dependency
  else {
    panic!("unexpected dependency");
  };
  assert_eq!(*actual_name, name);
  assert_eq!(Lexer::slice_range(input, range).unwrap(), name);
}

fn assert_local_counter_style_decl_dependency(input: &str, dependency: &Dependency, name: &str) {
  let Dependency::LocalCounterStyleDecl {
    name: actual_name,
    range,
  } = dependency
  else {
    panic!("unexpected dependency");
  };
  assert_eq!(*actual_name, name);
  assert_eq!(Lexer::slice_range(input, range).unwrap(), name);
}

fn assert_local_counter_style_dependency(input: &str, dependency: &Dependency, name: &str) {
  let Dependency::LocalCounterStyle {
    name: actual_name,
    range,
  } = dependency
  else {
    panic!("unexpected dependency");
  };
  assert_eq!(*actual_name, name);
  assert_eq!(Lexer::slice_range(input, range).unwrap(), name);
}

fn assert_local_font_palette_decl_dependency(input: &str, dependency: &Dependency, name: &str) {
  let Dependency::LocalFontPaletteDecl {
    name: actual_name,
    range,
  } = dependency
  else {
    panic!("unexpected dependency");
  };
  assert_eq!(*actual_name, name);
  assert_eq!(
    Lexer::slice_range(input, range).unwrap(),
    format!("--{}", name)
  );
}

fn assert_local_font_palette_dependency(input: &str, dependency: &Dependency, name: &str) {
  let Dependency::LocalFontPalette {
    name: actual_name,
    range,
  } = dependency
  else {
    panic!("unexpected dependency");
  };
  assert_eq!(*actual_name, name);
  assert_eq!(
    Lexer::slice_range(input, range).unwrap(),
    format!("--{}", name)
  );
}

fn assert_local_container_dependency(input: &str, dependency: &Dependency, name: &str) {
  let Dependency::LocalContainer {
    name: actual_name,
    range,
  } = dependency
  else {
    panic!("unexpected dependency");
  };
  assert_eq!(*actual_name, name);
  assert_eq!(Lexer::slice_range(input, range).unwrap(), name);
}

fn assert_local_container_decl_dependency(input: &str, dependency: &Dependency, name: &str) {
  let Dependency::LocalContainerDecl {
    name: actual_name,
    range,
  } = dependency
  else {
    panic!("unexpected dependency");
  };
  assert_eq!(*actual_name, name);
  assert_eq!(Lexer::slice_range(input, range).unwrap(), name);
}

fn assert_local_function_dependency(input: &str, dependency: &Dependency, name: &str) {
  let Dependency::LocalFunction {
    name: actual_name,
    range,
  } = dependency
  else {
    panic!("unexpected dependency");
  };
  assert_eq!(*actual_name, name);
  assert_eq!(
    Lexer::slice_range(input, range).unwrap(),
    format!("--{}", name)
  );
}

fn assert_local_function_decl_dependency(input: &str, dependency: &Dependency, name: &str) {
  let Dependency::LocalFunctionDecl {
    name: actual_name,
    range,
  } = dependency
  else {
    panic!("unexpected dependency");
  };
  assert_eq!(*actual_name, name);
  assert_eq!(
    Lexer::slice_range(input, range).unwrap(),
    format!("--{}", name)
  );
}

fn assert_local_grid_dependency(input: &str, dependency: &Dependency, name: &str) {
  let Dependency::LocalGrid {
    name: actual_name,
    range,
  } = dependency
  else {
    panic!("unexpected dependency");
  };
  assert_eq!(*actual_name, name);
  assert_eq!(Lexer::slice_range(input, range).unwrap(), name);
}

fn assert_local_grid_decl_dependency(input: &str, dependency: &Dependency, name: &str) {
  let Dependency::LocalGridDecl {
    name: actual_name,
    range,
  } = dependency
  else {
    panic!("unexpected dependency");
  };
  assert_eq!(*actual_name, name);
  assert_eq!(Lexer::slice_range(input, range).unwrap(), name);
}

fn assert_composes_dependency(
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
    SmallVec::<[&str; 2]>::from_iter(local_classes.split(' '))
  );
  assert_eq!(
    context
      .composes_names(*actual_names)
      .iter()
      .copied()
      .collect::<SmallVec<[&str; 2]>>(),
    SmallVec::<[&str; 2]>::from_iter(names.split(' '))
  );
  assert_eq!(*actual_from, from);
  assert_eq!(Lexer::slice_range(input, range).unwrap(), range_content);
}

fn assert_replace_dependency(
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
  assert_eq!(Lexer::slice_range(input, range).unwrap(), range_content);
}

fn assert_charset_dependency(
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
  assert_eq!(Lexer::slice_range(input, range).unwrap(), range_content);
}

fn assert_icss_import_from_dependency(_input: &str, dependency: &Dependency, path: &str) {
  let Dependency::ICSSImportFrom { path: actual_path } = dependency else {
    panic!("unexpected dependency");
  };
  assert_eq!(*actual_path, path);
}

fn assert_icss_import_value_dependency(
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

fn assert_icss_export_value_dependency(
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

fn assert_icss_symbol_dependency(
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
  assert_eq!(Lexer::slice_range(input, range).unwrap(), range_content);
}

#[test]
fn empty() {
  let (dependencies, warnings) = collect_dependencies("", Mode::Css);
  assert!(warnings.is_empty());
  assert!(dependencies.is_empty());
}

#[test]
fn dependency_urls_and_imports_preserve_css_escapes() {
  let input = r#"@charset "UTF\2d 8";"#;
  let (dependencies, warnings) = collect_dependencies(input, Mode::Css);
  assert!(warnings.is_empty());
  assert_charset_dependency(input, &dependencies[0], r"UTF\2d 8", input);

  let input = r"body { background: url(a\20 b.png) }";
  let (dependencies, warnings) = collect_dependencies(input, Mode::Css);
  assert!(warnings.is_empty());
  assert_url_dependency(
    input,
    &dependencies[0],
    r"a\20 b.png",
    UrlRangeKind::Function,
    r"url(a\20 b.png)",
  );

  let input = r#"@import "theme\20 dark.css";"#;
  let (dependencies, warnings) = collect_dependencies(input, Mode::Css);
  assert!(warnings.is_empty());
  assert_import_dependency(
    input,
    import_dependency(&dependencies, 0),
    r"theme\20 dark.css",
    None,
    None,
    None,
    input,
  );

  let input = r"@import f\6f o;";
  let (dependencies, warnings) = collect_dependencies(input, Mode::Css);
  assert!(warnings.is_empty());
  assert_icss_import_url_dependency(input, &dependencies[0], r"f\6f o", input, r"f\6f o");
}

#[test]
fn dependency_css_module_names_preserve_css_escapes() {
  let input = r".f\6f o {} #i\64{}";
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());

  let Dependency::LocalClass { name, range, .. } = &dependencies[0] else {
    panic!("unexpected dependency");
  };
  assert_eq!(*name, r".f\6f o");
  assert_eq!(Lexer::slice_range(input, range), Some(r".f\6f o"));

  let Dependency::LocalId { name, range, .. } = &dependencies[1] else {
    panic!("unexpected dependency");
  };
  assert_eq!(*name, r"#i\64");
  assert_eq!(Lexer::slice_range(input, range), Some(r"#i\64"));

  let input = r"@keyframes sl\69 de {} .x { animation-name: sl\69 de; }";
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  let declaration = dependencies
    .iter()
    .find(|dependency| matches!(dependency, Dependency::LocalKeyframesDecl { .. }))
    .unwrap();
  let Dependency::LocalKeyframesDecl { name, range } = declaration else {
    unreachable!();
  };
  assert_eq!(*name, r"sl\69 de");
  assert_eq!(Lexer::slice_range(input, range), Some(r"sl\69 de"));
  let usage = dependencies
    .iter()
    .find(|dependency| matches!(dependency, Dependency::LocalKeyframes { .. }))
    .unwrap();
  let Dependency::LocalKeyframes { name, range } = usage else {
    unreachable!();
  };
  assert_eq!(*name, r"sl\69 de");
  assert_eq!(Lexer::slice_range(input, range), Some(r"sl\69 de"));

  let input = r#".x { --v\61 r: red; color: var(--v\61 r from "theme\20 dark.css"); }"#;
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  let declaration = dependencies
    .iter()
    .find(|dependency| matches!(dependency, Dependency::LocalVarDecl { .. }))
    .unwrap();
  let Dependency::LocalVarDecl { name, range } = declaration else {
    unreachable!();
  };
  assert_eq!(*name, r"v\61 r");
  assert_eq!(Lexer::slice_range(input, range), Some(r"--v\61 r"));
  let usage = dependencies
    .iter()
    .find(|dependency| matches!(dependency, Dependency::LocalVar { .. }))
    .unwrap();
  let Dependency::LocalVar { name, from, range } = usage else {
    unreachable!();
  };
  assert_eq!(*name, r"v\61 r");
  assert_eq!(*from, Some(r#""theme\20 dark.css""#));
  assert_eq!(Lexer::slice_range(input, range), Some(r"--v\61 r"));
}

#[test]
fn dependency_composes_and_icss_strings_preserve_css_escapes() {
  let input = r#".\66 oo { composes: b\61 r from "theme\20 dark.css"; }"#;
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  let composes = dependencies
    .iter()
    .find(|dependency| matches!(dependency, Dependency::Composes { .. }))
    .unwrap();
  let Dependency::Composes {
    local_classes,
    names,
    from,
    range,
  } = composes
  else {
    unreachable!();
  };
  assert_eq!(
    dependencies.composes_local_classes(*local_classes),
    [r"\66 oo"]
  );
  assert_eq!(dependencies.composes_names(*names), [r"b\61 r"]);
  assert_eq!(*from, Some(r#""theme\20 dark.css""#));
  assert_eq!(
    Lexer::slice_range(input, range),
    Some(r#"b\61 r from "theme\20 dark.css""#)
  );

  let input = r#":import("./theme\20 dark.css") { l\6f cal: rem\6f te; }
.x { color: l\6f cal; }"#;
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_icss_import_from_dependency(input, &dependencies[0], r#""./theme\20 dark.css""#);
  assert_icss_import_value_dependency(input, &dependencies[1], r"l\6f cal", r"rem\6f te");
  let symbol = dependencies
    .iter()
    .find(|dependency| matches!(dependency, Dependency::ICSSSymbol { .. }))
    .unwrap();
  assert_icss_symbol_dependency(input, symbol, r"l\6f cal", r"l\6f cal");

  let input = r":export { f\6f o: red; } .x { color: f\6f o; }";
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  let export = dependencies
    .iter()
    .find(|dependency| matches!(dependency, Dependency::ICSSExportValue { .. }))
    .unwrap();
  assert_icss_export_value_dependency(input, export, r"f\6f o", "red");
  let symbol = dependencies
    .iter()
    .find(|dependency| matches!(dependency, Dependency::ICSSSymbol { .. }))
    .unwrap();
  assert_icss_symbol_dependency(input, symbol, r"f\6f o", r"f\6f o");
}

#[test]
fn url() {
  let input = indoc! {r#"
        body {
            --a: url("./logo.png");
            background: url(
                https://example\2f4a8f.com\
        /image.png
            )
        }
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Css);
  assert!(warnings.is_empty());
  assert_url_dependency(
    input,
    &dependencies[0],
    "./logo.png",
    UrlRangeKind::String,
    "\"./logo.png\"",
  );
  assert_url_dependency(
    input,
    &dependencies[1],
    r"https://example\2f4a8f.com\
/image.png",
    UrlRangeKind::Function,
    "url(\n        https://example\\2f4a8f.com\\\n/image.png\n    )",
  );
  assert_eq!(dependencies.len(), 2);
}

#[test]
fn url_2() {
  let input = "body{background-image:url(./img.png)}";
  let (dependencies, warnings) = collect_dependencies(input, Mode::Css);
  assert!(warnings.is_empty());
  assert_url_dependency(
    input,
    &dependencies[0],
    "./img.png",
    UrlRangeKind::Function,
    "url(./img.png)",
  );
}

#[test]
fn url_3() {
  let input = r#"body{content: "\f101";background-image:url(./img.png)}"#;
  let (dependencies, warnings) = collect_dependencies(input, Mode::Css);
  assert!(warnings.is_empty());
  assert_url_dependency(
    input,
    &dependencies[0],
    "./img.png",
    UrlRangeKind::Function,
    "url(./img.png)",
  );
}

#[test]
fn url_4() {
  let input = r#"body{content: "\f\"101";background-image:url(./img.png)}"#;
  let (dependencies, warnings) = collect_dependencies(input, Mode::Css);
  assert!(warnings.is_empty());
  assert_url_dependency(
    input,
    &dependencies[0],
    "./img.png",
    UrlRangeKind::Function,
    "url(./img.png)",
  );
}

#[test]
fn duplicate_url() {
  let input = indoc! {r#"
        @import url(./a.css) url(./a.css);
        @import url(./a.css) url("./a.css");
        @import url("./a.css") url(./a.css);
        @import url("./a.css") url("./a.css");
    "#};
  let (_, warnings) = collect_dependencies(input, Mode::Css);
  assert_warning(input, &warnings[0], "@import url(./a.css) url(./a.css)");
  assert_warning(input, &warnings[1], "@import url(./a.css) url(\"./a.css\"");
  assert_warning(input, &warnings[2], "@import url(\"./a.css\") url(./a.css)");
  assert_warning(
    input,
    &warnings[3],
    "@import url(\"./a.css\") url(\"./a.css\"",
  );
}

#[test]
fn icss_import_url_value() {
  let input = "@import importPath;";
  let (dependencies, warnings) = collect_dependencies(input, Mode::Css);
  assert!(warnings.is_empty());
  assert_icss_import_url_dependency(
    input,
    &dependencies[0],
    "importPath",
    "@import importPath;",
    "importPath",
  );
  assert_eq!(dependencies.len(), 1);
}

#[test]
fn icss_import_url_value_with_comments_and_spacing() {
  let input = "@import/* before */ importPath /* after */;";
  let (dependencies, warnings) = collect_dependencies(input, Mode::Css);
  assert!(warnings.is_empty());
  assert_icss_import_url_dependency(input, &dependencies[0], "importPath", input, "importPath");
  assert_eq!(dependencies.len(), 1);
}

#[test]
fn invalid_icss_import_url_value() {
  let input = "@import importPath screen;";
  let (dependencies, warnings) = collect_dependencies(input, Mode::Css);
  assert!(dependencies.is_empty());
  assert_warning(input, &warnings[0], "@import importPath screen;");
}

#[test]
fn invalid_icss_import_url_value_with_function() {
  let input = "@import importPath supports(display: grid;";
  let (dependencies, warnings) = collect_dependencies(input, Mode::Css);
  assert!(dependencies.is_empty());
  assert_warning(
    input,
    &warnings[0],
    "@import importPath supports(display: grid;",
  );
}

#[test]
fn invalid_icss_import_url_value_before_string_url() {
  let input = r#"@import importPath "style.css";"#;
  let (dependencies, warnings) = collect_dependencies(input, Mode::Css);
  assert!(dependencies.is_empty());
  assert_warning(input, &warnings[0], r#""style.css""#);
}

#[test]
fn import_url_function_is_not_icss_import_url_value() {
  let input = "@import url(./style.css);";
  let (dependencies, warnings) = collect_dependencies(input, Mode::Css);
  assert!(warnings.is_empty());
  assert_import_dependency(
    input,
    import_dependency(&dependencies, 0),
    "./style.css",
    None,
    None,
    None,
    "@import url(./style.css);",
  );
  assert_eq!(dependencies.len(), 1);
}

#[test]
fn selector_after_import_uses_selector_scan_context() {
  let input = r#"@import "./style.css"; .after {}"#;
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);

  assert!(warnings.is_empty());
  assert_import_dependency(
    input,
    import_dependency(&dependencies, 0),
    "./style.css",
    None,
    None,
    None,
    r#"@import "./style.css";"#,
  );
  assert_local_class_dependency(input, &dependencies[1], ".after", false);
  assert_eq!(dependencies.len(), 2);
}

#[test]
fn not_preceded_at_import() {
  let input = indoc! {r#"
        body {}
        @import url(./a.css);
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Css);
  assert!(dependencies.is_empty());
  assert_warning(input, &warnings[0], "@import");
}

#[test]
fn url_string() {
  let input = indoc! {r#"
        body {
            a: url("https://example\2f4a8f.com\
            /image.png");
            b: image-set(
                "image1.png" 1x,
                "image2.png" 2x
            );
            c: image-set(
                url(image1.avif) type("image/avif"),
                url("image2.jpg") type("image/jpeg")
            );
        }
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Css);
  assert!(warnings.is_empty());
  assert_url_dependency(
    input,
    &dependencies[0],
    r"https://example\2f4a8f.com\
    /image.png",
    UrlRangeKind::String,
    "\"https://example\\2f4a8f.com\\\n    /image.png\"",
  );
  assert_url_dependency(
    input,
    &dependencies[1],
    "image1.png",
    UrlRangeKind::Function,
    "\"image1.png\"",
  );
  assert_url_dependency(
    input,
    &dependencies[2],
    "image2.png",
    UrlRangeKind::Function,
    "\"image2.png\"",
  );
  assert_url_dependency(
    input,
    &dependencies[3],
    "image1.avif",
    UrlRangeKind::Function,
    "url(image1.avif)",
  );
  assert_url_dependency(
    input,
    &dependencies[4],
    "image2.jpg",
    UrlRangeKind::String,
    "\"image2.jpg\"",
  );
}

#[test]
fn empty_url() {
  let input = indoc! {r#"
        @import url();
        @import url("");
        body {
            a: url();
            b: url("");
            c: image-set(); // not an dependency
            d: image-set("");
            e: image-set(url());
            f: image-set(url(""));
        }
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Css);
  assert!(warnings.is_empty());
  assert_import_dependency(
    input,
    import_dependency(&dependencies, 0),
    "",
    None,
    None,
    None,
    "@import url();",
  );
  assert_import_dependency(
    input,
    import_dependency(&dependencies, 1),
    "",
    None,
    None,
    None,
    "@import url(\"\");",
  );
  assert_url_dependency(input, &dependencies[2], "", UrlRangeKind::Function, "url()");
  assert_url_dependency(input, &dependencies[3], "", UrlRangeKind::String, "\"\"");
  assert_url_dependency(input, &dependencies[4], "", UrlRangeKind::Function, "\"\"");
  assert_url_dependency(input, &dependencies[5], "", UrlRangeKind::Function, "url()");
  assert_url_dependency(input, &dependencies[6], "", UrlRangeKind::String, "\"\"");
}

#[test]
fn expect_url() {
  let input = indoc! {r#"
        @import ;
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Css);
  assert!(dependencies.is_empty());
  assert_warning(input, &warnings[0], "@import ;");
}

#[test]
#[rustfmt::skip]
fn import() {
  let input = indoc! {r#"
      @import 'https://example\2f4a8f.com\
      /style.css';
      @import url(https://example\2f4a8f.com\
      /style.css);
      @import url('https://example\2f4a8f.com\
      /style.css') /* */;
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Css);
  assert!(warnings.is_empty());
  assert_import_dependency(
    input,
    import_dependency(&dependencies, 0),
    r"https://example\2f4a8f.com\
/style.css",
    None,
    None,
    None,
    "@import 'https://example\\2f4a8f.com\\\n/style.css';",
  );
  assert_import_dependency(
    input,
    import_dependency(&dependencies, 1),
    r"https://example\2f4a8f.com\
/style.css",
    None,
    None,
    None,
    "@import url(https://example\\2f4a8f.com\\\n/style.css);",
  );
  assert_import_dependency(
    input,
    import_dependency(&dependencies, 2),
    r"https://example\2f4a8f.com\
/style.css",
    None,
    None,
    None,
    "@import url('https://example\\2f4a8f.com\\\n/style.css') /* */;",
  );
}

#[test]
fn unexpected_semicolon_in_supports() {
  let input = indoc! {r#"
        @import "style.css" supports(display: flex; display: grid);
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Css);
  assert_import_dependency(
    input,
    import_dependency(&dependencies, 0),
    "style.css",
    None,
    None,
    Some(" supports(display: flex"),
    "@import \"style.css\" supports(display: flex;",
  );
  assert_warning(input, &warnings[0], ";");
}

#[test]
fn unexpected_semicolon_import_url_string() {
  let input = indoc! {r#"
        @import url("style.css";);
        @import url("style.css" layer;);
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Css);
  assert!(dependencies.is_empty());
  assert_warning(input, &warnings[0], ";");
  assert_warning(input, &warnings[1], ";");
}

#[test]
fn expected_before() {
  let input = indoc! {r#"
        @import layer supports(display: flex) "style.css";
        @import supports(display: flex) "style.css";
        @import layer "style.css";
        @import "style.css" supports(display: flex) layer;
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Css);
  assert!(dependencies.is_empty());
  assert_warning(input, &warnings[0], "\"style.css\"");
  assert_warning(input, &warnings[1], "\"style.css\"");
  assert_warning(input, &warnings[2], "\"style.css\"");
  assert_warning(input, &warnings[3], "layer");
}

#[test]
fn import_media() {
  let input = indoc! {r#"
        @import url("style.css") screen and (orientation: portrait);
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Css);
  assert!(warnings.is_empty());
  assert_import_dependency(
    input,
    import_dependency(&dependencies, 0),
    "style.css",
    None,
    None,
    Some(" screen and (orientation: portrait)"),
    "@import url(\"style.css\") screen and (orientation: portrait);",
  );
}

#[test]
fn import_attributes() {
  let input = indoc! {r#"
        @import url("style.css") layer;
        @import url("style.css") supports();
        @import url("style.css") print;
        @import url("style.css") layer supports() /* comments */;
        @import url("style.css") layer(default) supports(not (display: grid) and (display: flex)) print, /* comments */ screen and (orientation: portrait);
        @import URL("style.css") LAYER(DEFAULT) SUPPORTS(DISPLAY: FLEX) SCREEN AND (MIN-WIDTH: 400PX);
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Css);
  assert!(warnings.is_empty());
  assert_import_dependency(
    input,
    import_dependency(&dependencies, 0),
    "style.css",
    Some(""),
    None,
    None,
    "@import url(\"style.css\") layer;",
  );
  assert_import_dependency(
    input,
    import_dependency(&dependencies, 1),
    "style.css",
    None,
    Some(""),
    None,
    "@import url(\"style.css\") supports();",
  );
  assert_import_dependency(
    input,
    import_dependency(&dependencies, 2),
    "style.css",
    None,
    None,
    Some(" print"),
    "@import url(\"style.css\") print;",
  );
  assert_import_dependency(
    input,
    import_dependency(&dependencies, 3),
    "style.css",
    Some(""),
    Some(""),
    None,
    "@import url(\"style.css\") layer supports() /* comments */;",
  );
  assert_import_dependency(
    input,
    import_dependency(&dependencies, 4),
    "style.css",
    Some("default"),
    Some("not (display: grid) and (display: flex)"),
    Some(" print, /* comments */ screen and (orientation: portrait)"),
    "@import url(\"style.css\") layer(default) supports(not (display: grid) and (display: flex)) print, /* comments */ screen and (orientation: portrait);",
  );
  assert_import_dependency(
    input,
    import_dependency(&dependencies, 5),
    "style.css",
    Some("DEFAULT"),
    Some("DISPLAY: FLEX"),
    Some(" SCREEN AND (MIN-WIDTH: 400PX)"),
    "@import URL(\"style.css\") LAYER(DEFAULT) SUPPORTS(DISPLAY: FLEX) SCREEN AND (MIN-WIDTH: 400PX);",
  );
}

#[test]
fn css_modules_pseudo_1() {
  let input = ".localA :global .global-b .global-c :local(.localD.localE) .global-d";
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_local_class_dependency(input, &dependencies[0], ".localA", false);
  assert_replace_dependency(input, &dependencies[1], "", ":global ");
  assert_replace_dependency(input, &dependencies[2], "", ":local(");
  assert_local_class_dependency(input, &dependencies[3], ".localD", true);
  assert_local_class_dependency(input, &dependencies[4], ".localE", true);
  assert_replace_dependency(input, &dependencies[5], "", ")");
}

#[test]
fn css_modules_pseudo_2() {
  let input = indoc! {r#"
        :global .a :local .b :global .c {}
        .d #e {}
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_replace_dependency(input, &dependencies[0], "", ":global ");
  assert_replace_dependency(input, &dependencies[1], "", ":local ");
  assert_local_class_dependency(input, &dependencies[2], ".b", true);
  assert_replace_dependency(input, &dependencies[3], "", ":global ");
  assert_local_class_dependency(input, &dependencies[4], ".d", false);
  assert_local_id_dependency(input, &dependencies[5], "#e", false);
  assert_eq!(dependencies.len(), 6);
}

#[test]
fn css_modules_local_class_after_comment_keeps_dependencies() {
  let input = ":local/** comment **/.class {}";
  let (dependencies, warnings) = collect_dependencies(input, Mode::Global);
  assert_eq!(
    warnings.iter().map(ToString::to_string).collect::<Vec<_>>(),
    ["Missing trailing whitespace"]
  );
  assert_replace_dependency(input, &dependencies[0], "", ":local");
  assert_local_class_dependency(input, &dependencies[1], ".class", true);
  assert_eq!(dependencies.len(), 2);
}

#[test]
fn css_modules_mode_replace_stops_before_comment_after_white_space() {
  let input = ":local /** first **//** second **/.class {}";
  let (dependencies, warnings) = collect_dependencies(input, Mode::Global);

  assert!(warnings.is_empty());
  assert_replace_dependency(input, &dependencies[0], "", ":local ");
  assert_local_class_dependency(input, &dependencies[1], ".class", true);
  assert_eq!(dependencies.len(), 2);
}

#[test]
fn css_modules_local_id_after_comment_keeps_dependencies() {
  let input = ":local/** comment **/#id {}";
  let (dependencies, warnings) = collect_dependencies(input, Mode::Global);
  assert_eq!(
    warnings.iter().map(ToString::to_string).collect::<Vec<_>>(),
    ["Missing trailing whitespace"]
  );
  assert_replace_dependency(input, &dependencies[0], "", ":local");
  assert_local_id_dependency(input, &dependencies[1], "#id", true);
  assert_eq!(dependencies.len(), 2);
}

#[test]
fn css_modules_global_class_after_comment_keeps_dependencies() {
  let input = ":global/** comment **/.class {}";
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert_eq!(
    warnings.iter().map(ToString::to_string).collect::<Vec<_>>(),
    ["Missing trailing whitespace"]
  );
  assert_replace_dependency(input, &dependencies[0], "", ":global");
  assert_eq!(dependencies.len(), 1);
}

#[test]
fn css_modules_local_block_after_comment_keeps_dependencies() {
  let input = ":local/** comment **/{ color: red; }";
  let (dependencies, warnings) = collect_dependencies(input, Mode::Global);
  assert_eq!(
    warnings.iter().map(ToString::to_string).collect::<Vec<_>>(),
    ["Missing trailing whitespace"]
  );
  assert_replace_dependency(input, &dependencies[0], "", ":local");
  assert_eq!(dependencies.len(), 1);
}

#[test]
fn css_mode_does_not_recover_comment_separated_mode_dependencies() {
  let input = ":local/** comment **/.class {}";
  let (dependencies, warnings) = collect_dependencies(input, Mode::Css);
  assert!(dependencies.is_empty());
  assert!(warnings.is_empty());
}

const RSPACK_NESTED_COMMENT_SEPARATED_MODES: &str = r#"
.no-space {
    :global.class-no-space {
        color: red;
    }

    :global/** test **/.class {
        color: red;
    }

    :local.class {
        color: red;
    }

    :local/** test **/.class {
        color: red;
    }

    :local/** test **/#hash {
        color: red;
    }

    :local/** test **/{
        color: red;
    }
}
"#;

#[test]
fn rspack_nested_comment_separated_modes_recover_selector_dependencies() {
  let input = RSPACK_NESTED_COMMENT_SEPARATED_MODES;
  let (dependencies, _) = collect_dependencies(input, Mode::Local);

  let local_dependencies = dependencies
    .iter()
    .filter_map(|dependency| match dependency {
      Dependency::LocalClass { name, explicit, .. }
      | Dependency::LocalId { name, explicit, .. } => Some((*name, *explicit)),
      _ => None,
    })
    .collect::<Vec<_>>();
  assert_eq!(
    local_dependencies,
    [
      (".no-space", false),
      (".class", true),
      (".class", true),
      ("#hash", true),
    ]
  );

  let replacements = dependencies
    .iter()
    .filter_map(|dependency| match dependency {
      Dependency::Replace { range, .. } => Lexer::slice_range(input, range),
      _ => None,
    })
    .collect::<Vec<_>>();
  assert_eq!(
    replacements,
    [":global", ":global", ":local", ":local", ":local", ":local"]
  );
}

#[test]
fn rspack_nested_comment_separated_modes_warn_in_source_order() {
  let input = RSPACK_NESTED_COMMENT_SEPARATED_MODES;
  let (_, warnings) = collect_dependencies(input, Mode::Local);

  assert_eq!(
    warnings.iter().map(ToString::to_string).collect::<Vec<_>>(),
    [
      "Missing trailing whitespace",
      "Missing trailing whitespace",
      "Missing trailing whitespace",
      "Missing trailing whitespace",
      "Missing trailing whitespace",
      "Missing trailing whitespace",
    ]
  );
  assert_eq!(
    warnings
      .iter()
      .map(|warning| Lexer::slice_range(input, warning.range()).unwrap())
      .collect::<Vec<_>>(),
    [":global", ":global", ":local", ":local", ":local", ":local"]
  );
}

#[test]
fn missing_whitespace_mode_dependencies_are_emitted_by_the_main_lexer() {
  let mut warnings = Vec::new();
  let dependencies = lex_dependencies(
    RSPACK_NESTED_COMMENT_SEPARATED_MODES,
    Mode::Local,
    |_| {},
    |warning| warnings.push(warning),
  );

  assert_eq!(
    (dependencies, warnings),
    collect_dependencies(RSPACK_NESTED_COMMENT_SEPARATED_MODES, Mode::Local)
  );
}

#[test]
fn mode_pseudo_syntax_in_declaration_value_is_not_scanned_as_selector() {
  for input in [
    ".before { custom: :local/** comment **/.value; } .after {}",
    ".before { custom: :local/** comment **/.value } .after {}",
  ] {
    let (dependencies, warnings) = collect_dependencies(input, Mode::Local);

    assert!(warnings.is_empty());
    assert_local_class_dependency(input, &dependencies[0], ".before", false);
    assert_local_class_dependency(input, &dependencies[1], ".after", false);
    assert_eq!(dependencies.len(), 2);
  }
}

#[test]
fn comment_separated_declaration_value_is_not_scanned_as_selector() {
  let input = ".before { color: /* comment */ red; } .after {}";
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);

  assert!(warnings.is_empty());
  assert_local_class_dependency(input, &dependencies[0], ".before", false);
  assert_local_class_dependency(input, &dependencies[1], ".after", false);
  assert_eq!(dependencies.len(), 2);
}

#[test]
fn rspack_mixed_nested_selector_recovers_dependencies() {
  let input = indoc! {r#"
        :global .global-foo, :local .bar {
            :local .local-in-global {
                color: blue;
            }

            @media screen {
                :global .my-global-class-again,
                :local .my-global-class-again {
                    color: red;
                }
            }
        }
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert_eq!(warnings.len(), 2);
  assert!(
    warnings
      .iter()
      .all(|warning| matches!(warning.kind(), WarningKind::InconsistentModeResult))
  );
  assert_replace_dependency(input, &dependencies[5], "", ":global ");
  assert_replace_dependency(input, &dependencies[6], "", ":local ");
  assert_local_class_dependency(input, &dependencies[7], ".my-global-class-again", true);
  assert_eq!(dependencies.len(), 8);
}

#[test]
fn css_modules_uppercase_pseudo_does_not_leave_balanced_stack_dirty() {
  let input = indoc! {r#"
        :GLOBAL .globalUpperCase :LOCAL .localUpperCase {
            color: yellow;
        }
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_replace_dependency(input, &dependencies[0], "", ":GLOBAL ");
  assert_replace_dependency(input, &dependencies[1], "", ":LOCAL ");
  assert_local_class_dependency(input, &dependencies[2], ".localUpperCase", true);
  assert_eq!(dependencies.len(), 3);
}

#[test]
fn css_modules_pseudo_3() {
  let input = ".a:not(:global .b:not(.c:not(:global .d) .e) .f).g {}";
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_local_class_dependency(input, &dependencies[0], ".a", false);
  assert_replace_dependency(input, &dependencies[1], "", ":global ");
  assert_replace_dependency(input, &dependencies[2], "", ":global ");
  assert_local_class_dependency(input, &dependencies[3], ".g", false);
  assert_eq!(dependencies.len(), 4);
}

#[test]
fn css_modules_pseudo_4() {
  let input = ".a:not(:global .b:not(:local .c:not(:global .d) .e) .f).g {}";
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_local_class_dependency(input, &dependencies[0], ".a", false);
  assert_replace_dependency(input, &dependencies[1], "", ":global ");
  assert_replace_dependency(input, &dependencies[2], "", ":local ");
  assert_local_class_dependency(input, &dependencies[3], ".c", true);
  assert_replace_dependency(input, &dependencies[4], "", ":global ");
  assert_local_class_dependency(input, &dependencies[5], ".e", true);
  assert_local_class_dependency(input, &dependencies[6], ".g", false);
  assert_eq!(dependencies.len(), 7);
}

#[test]
fn css_modules_pseudo_5() {
  let input = ":global(.a, .b) {}";
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_replace_dependency(input, &dependencies[0], "", ":global(");
  assert_replace_dependency(input, &dependencies[1], "", ")");
  assert_eq!(dependencies.len(), 2);
}

#[test]
fn css_modules_pseudo_6() {
  let input = ".a:local( .b ).c {}";
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_local_class_dependency(input, &dependencies[0], ".a", false);
  assert_replace_dependency(input, &dependencies[1], "", ":local( ");
  assert_local_class_dependency(input, &dependencies[2], ".b", true);
  assert_replace_dependency(input, &dependencies[3], "", " )");
  assert_local_class_dependency(input, &dependencies[4], ".c", false);
  assert_eq!(dependencies.len(), 5);
}

#[test]
fn css_modules_pseudo_7() {
  let input = "@charset \"UTF-8\";.a{}.b{}";
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_charset_dependency(input, &dependencies[0], "UTF-8", "@charset \"UTF-8\";");
  assert_local_class_dependency(input, &dependencies[1], ".a", false);
  assert_local_class_dependency(input, &dependencies[2], ".b", false);
  assert_eq!(dependencies.len(), 3);
}

#[test]
fn css_modules_charset_in_string_and_comment() {
  let input = ".a{content:'@charset \"UTF-8\";'}/*@charset \"UTF-8\";*/.b{}";
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_local_class_dependency(input, &dependencies[0], ".a", false);
  assert_local_class_dependency(input, &dependencies[1], ".b", false);
  assert_eq!(dependencies.len(), 2);
}

#[test]
fn charset_at_rule() {
  let input = "@charset \"UTF-8\";@charset 'iso-8859-1';";
  let (dependencies, warnings) = collect_dependencies(input, Mode::Css);
  assert!(warnings.is_empty());
  assert_charset_dependency(input, &dependencies[0], "UTF-8", "@charset \"UTF-8\";");
  assert_charset_dependency(
    input,
    &dependencies[1],
    "iso-8859-1",
    "@charset 'iso-8859-1';",
  );
  assert_eq!(dependencies.len(), 2);
}

#[test]
fn pure_mode_ignore_comment_suppresses_next_rule() {
  let input = indoc! {r#"
        /* cssmodules-pure-ignore */
        :global(.ignored) {
            color: pink;
        }

        .normal {
            color: black;
        }
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Pure);
  assert!(warnings.is_empty());
  assert_replace_dependency(input, &dependencies[0], "", ":global(");
  assert_replace_dependency(input, &dependencies[1], "", ")");
  assert_local_class_dependency(input, &dependencies[2], ".normal", false);
  assert_eq!(dependencies.len(), 3);
}

#[test]
fn pure_mode_ignore_comment_does_not_affect_following_rule() {
  let input = indoc! {r#"
        /* cssmodules-pure-ignore */
        :global(.ignored) {
            color: pink;
        }

        :global(.not-ignored) {
            color: red;
        }
    "#};
  let (_, warnings) = collect_dependencies(input, Mode::Pure);
  assert_eq!(warnings.len(), 1);
  assert!(
    Lexer::slice_range(input, warnings[0].range())
      .unwrap()
      .contains(":global(.not-ignored)")
  );
}

#[test]
fn pure_mode_ignore_comment_suppresses_nested_rule() {
  let input = indoc! {r#"
        .foo {
            /* cssmodules-pure-ignore */
            :global(.bar) {
                color: blue;
            }
        }
    "#};
  let (_, warnings) = collect_dependencies(input, Mode::Pure);
  assert!(warnings.is_empty());
}

#[test]
fn pure_mode_nested_ignore_comment_does_not_suppress_outer_rule() {
  let input = indoc! {r#"
        :global(.foo) {
            /* cssmodules-pure-ignore */
            :global(.bar) {
                color: blue;
            }
        }
    "#};
  let (_, warnings) = collect_dependencies(input, Mode::Pure);
  assert_eq!(warnings.len(), 1);
  assert!(
    Lexer::slice_range(input, warnings[0].range())
      .unwrap()
      .contains(":global(.foo)")
  );
}

#[test]
fn pure_mode_no_check_comment_suppresses_file() {
  let input = indoc! {r#"
        /* file leading comment */
        /* cssmodules-pure-no-check - needed for third party integration */

        body {
            color: red;
        }

        :global(.global-only) {
            color: green;
        }
    "#};
  let (_, warnings) = collect_dependencies(input, Mode::Pure);
  assert!(warnings.is_empty());
}

#[test]
fn css_modules_missing_white_space_1() {
  let input = ".a:global,:global .b {}";
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_local_class_dependency(input, &dependencies[0], ".a", false);
  assert_replace_dependency(input, &dependencies[1], "", ":global");
  assert_replace_dependency(input, &dependencies[2], "", ":global ");
  assert_eq!(dependencies.len(), 3);
}

#[test]
fn css_modules_missing_white_space_2() {
  let input = ".a{}:global .b{}";
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_local_class_dependency(input, &dependencies[0], ".a", false);
  assert_replace_dependency(input, &dependencies[1], "", ":global ");
  assert_eq!(dependencies.len(), 2);
}

#[test]
fn css_modules_missing_white_space_3() {
  let input = ":global .a {}";
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_replace_dependency(input, &dependencies[0], "", ":global ");
  assert_eq!(dependencies.len(), 1);
}

#[test]
fn css_modules_missing_white_space_4() {
  let input = ".a:not(:global .b) {}";
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_local_class_dependency(input, &dependencies[0], ".a", false);
  assert_replace_dependency(input, &dependencies[1], "", ":global ");
  assert_eq!(dependencies.len(), 2);
}

#[test]
fn css_modules_missing_white_space_5() {
  let input = ".a:not(.b :global) {}";
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_local_class_dependency(input, &dependencies[0], ".a", false);
  assert_local_class_dependency(input, &dependencies[1], ".b", false);
  assert_replace_dependency(input, &dependencies[2], "", ":global");
  assert_eq!(dependencies.len(), 3);
}

#[test]
fn css_modules_missing_white_space_6() {
  let input = ".a :global,.b :global {}";
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_local_class_dependency(input, &dependencies[0], ".a", false);
  assert_replace_dependency(input, &dependencies[1], "", ":global");
  assert_local_class_dependency(input, &dependencies[2], ".b", false);
  assert_replace_dependency(input, &dependencies[3], "", ":global ");
  assert_eq!(dependencies.len(), 4);
}

#[test]
fn css_modules_missing_white_space_7() {
  let input = ".a :global{}";
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_local_class_dependency(input, &dependencies[0], ".a", false);
  assert_replace_dependency(input, &dependencies[1], "", ":global");
  assert_eq!(dependencies.len(), 2);
}

#[test]
fn css_modules_missing_white_space_8() {
  let input = ".a:global {}";
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_local_class_dependency(input, &dependencies[0], ".a", false);
  assert_replace_dependency(input, &dependencies[1], "", ":global ");
  assert_eq!(dependencies.len(), 2);
}

#[test]
fn css_modules_missing_white_space_9() {
  let input = ".a:global ,:global .b {}";
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_local_class_dependency(input, &dependencies[0], ".a", false);
  assert_replace_dependency(input, &dependencies[1], "", ":global ");
  assert_replace_dependency(input, &dependencies[2], "", ":global ");
  assert_eq!(dependencies.len(), 3);
}

#[test]
fn css_modules_missing_white_space_10() {
  let input = ".a:not(.b:not(:global .c):local .d) {}";
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_local_class_dependency(input, &dependencies[0], ".a", false);
  assert_local_class_dependency(input, &dependencies[1], ".b", false);
  assert_replace_dependency(input, &dependencies[2], "", ":global ");
  assert_replace_dependency(input, &dependencies[3], "", ":local ");
  assert_local_class_dependency(input, &dependencies[4], ".d", true);
  assert_eq!(dependencies.len(), 5);
}

#[test]
fn css_modules_missing_white_space_11() {
  let input = "@media(max-width: 1240px){:global #a{}}";
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_replace_dependency(input, &dependencies[0], "", ":global ");
  assert_eq!(dependencies.len(), 1);
}

#[test]
fn css_modules_nesting() {
  let input = indoc! {r#"
        .nested {
            .nested-nested {
                color: red;
            }
        }
        .nested-at-rule {
            @media screen {
                .nested-nested-at-rule-deep {
                    color: red;
                }
            }
        }
        :global .nested2 {
            .nested2-nested {
                color: red;
            }
        }
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_local_class_dependency(input, &dependencies[0], ".nested", false);
  assert_local_class_dependency(input, &dependencies[1], ".nested-nested", false);
  assert_local_class_dependency(input, &dependencies[2], ".nested-at-rule", false);
  assert_local_class_dependency(
    input,
    &dependencies[3],
    ".nested-nested-at-rule-deep",
    false,
  );
  assert_replace_dependency(input, &dependencies[4], "", ":global ");
  assert_local_class_dependency(input, &dependencies[5], ".nested2-nested", false);
  assert_eq!(dependencies.len(), 6);
}

#[test]
fn css_modules_nested_mode_pseudo_class_restores_local_mode() {
  let input = indoc! {r#"
        .outer {
            :local .local, :global .global {}
            :global .foo, .bar {}
        }

        #id-foo {
            #id-bar {}
        }
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert_eq!(warnings.len(), 1);
  assert!(matches!(
    warnings[0].kind(),
    WarningKind::InconsistentModeResult
  ));
  assert_local_class_dependency(input, &dependencies[0], ".outer", false);
  assert_replace_dependency(input, &dependencies[1], "", ":local ");
  assert_local_class_dependency(input, &dependencies[2], ".local", true);
  assert_replace_dependency(input, &dependencies[3], "", ":global ");
  assert_replace_dependency(input, &dependencies[4], "", ":global ");
  assert_local_class_dependency(input, &dependencies[5], ".bar", false);
  assert_local_id_dependency(input, &dependencies[6], "#id-foo", false);
  assert_local_id_dependency(input, &dependencies[7], "#id-bar", false);
  assert_eq!(dependencies.len(), 8);
}

#[test]
fn css_modules_local_var_unexpected() {
  let input = indoc! {r#"
        .vars {
            color: var(local-color);
        }
    "#};
  let (_, warnings) = collect_dependencies(input, Mode::Local);
  assert_warning(input, &warnings[0], "lo");
}

#[test]
fn css_modules_local_var_1() {
  let input = indoc! {r#"
        .vars {
            color: var(--local-color, red);
            --local-color: red;
        }
        .globalVars :global {
            color: var(--global-color);
            --global-color: red;
        }
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_local_class_dependency(input, &dependencies[0], ".vars", false);
  assert_local_var_dependency(input, &dependencies[1], "local-color", None);
  assert_local_var_decl_dependency(input, &dependencies[2], "local-color");
  assert_local_class_dependency(input, &dependencies[3], ".globalVars", false);
  assert_replace_dependency(input, &dependencies[4], "", ":global ");
}

#[test]
fn css_modules_local_var_2() {
  let input = indoc! {r#"
        .bar {
            a: var(--color1 from "./b.css", red);
            b: var(--color2 from './b.css', red);
            c: var(--color3 from global, red);
        }
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_local_class_dependency(input, &dependencies[0], ".bar", false);
  assert_local_var_dependency(input, &dependencies[1], "color1", Some("\"./b.css\""));
  assert_local_var_dependency(input, &dependencies[2], "color2", Some("'./b.css'"));
  assert_local_var_dependency(input, &dependencies[3], "color3", Some("global"));
  assert_eq!(dependencies.len(), 4);
}

#[test]
fn css_modules_local_var_minified_1() {
  let input = "body{margin:0;font-family:var(--bs-body-font-family);}";
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_local_var_dependency(input, &dependencies[0], "bs-body-font-family", None);
}

#[test]
fn css_modules_local_var_minified_2() {
  let input = ".table-primary{--bs-table-color:#000;--bs-table-border-color:#a6b5cc;color:var(--bs-table-color);border-color:var(--bs-table-border-color)}";
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_local_class_dependency(input, &dependencies[0], ".table-primary", false);
  assert_local_var_decl_dependency(input, &dependencies[1], "bs-table-color");
  assert_local_var_decl_dependency(input, &dependencies[2], "bs-table-border-color");
  assert_local_var_dependency(input, &dependencies[3], "bs-table-color", None);
  assert_local_var_dependency(input, &dependencies[4], "bs-table-border-color", None);
}

#[test]
fn css_modules_property() {
  let input = indoc! {r#"
        @property --my-color {
            syntax: "<color>";
            inherits: false;
            initial-value: #c0ffee;
        }
        .class {
            color: var(--my-color);
        }
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_local_property_decl_dependency(input, &dependencies[0], "my-color");
  assert_local_class_dependency(input, &dependencies[1], ".class", false);
  assert_local_var_dependency(input, &dependencies[2], "my-color", None);
}

#[test]
fn css_modules_counter_style() {
  let input = indoc! {r#"
        @counter-style circles {
            symbols: Ⓐ Ⓑ Ⓒ;
        }
        ul {
            list-style: circles;
        }
        ol {
            list-style-type: none;
        }
        li {
            list-style-type: disc;
        }
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_local_counter_style_decl_dependency(input, &dependencies[0], "circles");
  assert_local_counter_style_dependency(input, &dependencies[1], "circles");
}

#[test]
fn css_modules_reserved_values_are_ascii_case_insensitive() {
  let input = indoc! {r#"
        .case-insensitive {
            animation: LiNeAr 1s;
            list-style: NoNe;
            container: NoRmAl;
            GrId-CoLuMn: SpAn 1;
        }
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_local_class_dependency(input, &dependencies[0], ".case-insensitive", false);
  assert_eq!(dependencies.len(), 1);
}

#[test]
fn css_modules_semantic_keywords_decode_css_escapes() {
  let input = indoc! {r#"
        .escaped {
            animation: l\69 near 1s;
            list-style: n\6f ne;
            grid-column: s\70 an 1;
            container: n\6f rmal;
            an\69 mation: fade 1s;
            color: v\61 r(--theme);
        }
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_local_class_dependency(input, &dependencies[0], ".escaped", false);
  assert_local_keyframes_dependency(input, &dependencies[1], "fade");
  assert_local_var_dependency(input, &dependencies[2], "theme", None);
  assert_eq!(dependencies.len(), 3);
}

#[test]
fn css_modules_dependency_functions_are_ascii_case_insensitive() {
  let input = indoc! {r#"
        .images {
            a: URL("a.png");
            b: IMAGE-SET("b.png" 1x);
            c: -WEBKIT-IMAGE-SET("c.png" 1x);
        }
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_local_class_dependency(input, &dependencies[0], ".images", false);
  assert_url_dependency(
    input,
    &dependencies[1],
    "a.png",
    UrlRangeKind::String,
    "\"a.png\"",
  );
  assert_url_dependency(
    input,
    &dependencies[2],
    "b.png",
    UrlRangeKind::Function,
    "\"b.png\"",
  );
  assert_url_dependency(
    input,
    &dependencies[3],
    "c.png",
    UrlRangeKind::Function,
    "\"c.png\"",
  );
  assert_eq!(dependencies.len(), 4);
}

#[test]
fn css_modules_vendor_prefixed_keywords_are_ascii_case_insensitive() {
  let input = indoc! {r#"
        @-WEBKIT-KEYFRAMES fade {}
        .animated {
            -WEBKIT-ANIMATION: fade 1s;
            -WEBKIT-ANIMATION-NAME: other;
        }
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_local_keyframes_decl_dependency(input, &dependencies[0], "fade");
  assert_local_class_dependency(input, &dependencies[1], ".animated", false);
  assert_local_keyframes_dependency(input, &dependencies[2], "fade");
  assert_local_keyframes_dependency(input, &dependencies[3], "other");
  assert_eq!(dependencies.len(), 4);
}

#[test]
fn css_modules_font_palette() {
  let input = indoc! {r#"
        @font-palette-values --Cooler {
            font-family: Bixa;
            base-palette: 1;
            override-colors: 1 #7EB7E4;
        }
        .foo {
            font-palette: --Cooler;
        }
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_local_font_palette_decl_dependency(input, &dependencies[0], "Cooler");
  assert_local_class_dependency(input, &dependencies[1], ".foo", false);
  assert_local_font_palette_dependency(input, &dependencies[2], "Cooler");
  assert_eq!(dependencies.len(), 3);
}

#[test]
fn css_modules_keyframes_unexpected() {
  let input = indoc! {r#"
        @keyframes $aaa {
            0% { color: var(--theme-color1); }
            100% { color: var(--theme-color2); }
        }
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert_warning(input, &warnings[0], "$a");
  assert_eq!(warnings.len(), 1);
  assert_local_var_dependency(input, &dependencies[0], "theme-color1", None);
  assert_local_var_dependency(input, &dependencies[1], "theme-color2", None);
  assert_eq!(dependencies.len(), 2);
}

#[test]
fn css_modules_keyframes_1() {
  let input = indoc! {r#"
        @keyframes localkeyframes {
            0% { color: var(--theme-color1); }
            100% { color: var(--theme-color2); }
        }
        @keyframes localkeyframes2 {
            0% { left: 0; }
            100% { left: 100px; }
        }
        .animation {
            animation-name: localkeyframes;
            animation: 3s ease-in 1s 2 reverse both paused localkeyframes, localkeyframes2;
            --theme-color1: red;
            --theme-color2: blue;
        }
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_local_keyframes_decl_dependency(input, &dependencies[0], "localkeyframes");
  assert_local_var_dependency(input, &dependencies[1], "theme-color1", None);
  assert_local_var_dependency(input, &dependencies[2], "theme-color2", None);
  assert_local_keyframes_decl_dependency(input, &dependencies[3], "localkeyframes2");
  assert_local_class_dependency(input, &dependencies[4], ".animation", false);
  assert_local_keyframes_dependency(input, &dependencies[5], "localkeyframes");
  assert_local_keyframes_dependency(input, &dependencies[6], "localkeyframes");
  assert_local_keyframes_dependency(input, &dependencies[7], "localkeyframes2");
  assert_local_var_decl_dependency(input, &dependencies[8], "theme-color1");
  assert_local_var_decl_dependency(input, &dependencies[9], "theme-color2");
  assert_eq!(dependencies.len(), 10);
}

#[test]
fn css_modules_keyframes_2() {
  let input = indoc! {r#"
        @keyframes slidein {
            from { width: 300%; }
            to { width: 100%; }
        }
        .class {
            --animation-name: slidein;
            animation:
                var(--animation-name) 3s,
                3s linear 1s infinite running env(slidein),
                3s linear env(slidein, var(--baz)) infinite running slidein;
        }
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_local_keyframes_decl_dependency(input, &dependencies[0], "slidein");
  assert_local_class_dependency(input, &dependencies[1], ".class", false);
  assert_local_var_decl_dependency(input, &dependencies[2], "animation-name");
  assert_local_var_dependency(input, &dependencies[3], "animation-name", None);
  assert_local_var_dependency(input, &dependencies[4], "baz", None);
  assert_local_keyframes_dependency(input, &dependencies[5], "slidein");
  assert_eq!(dependencies.len(), 6);
}

#[test]
fn css_modules_keyframes_3() {
  let input = "@keyframes :local(foo) {}";
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_replace_dependency(input, &dependencies[0], "", ":local(");
  assert_local_keyframes_decl_dependency(input, &dependencies[1], "foo");
  assert_replace_dependency(input, &dependencies[2], "", ")");
  assert_eq!(dependencies.len(), 3);
}

#[test]
fn css_modules_keyframes_4() {
  let input = indoc! {r#"
        @keyframes foo {}
        :local(.class) {
            animation-name: foo;
        }
        @keyframes :local(bar) {}
        :local .class2 {
            animation-name: bar;
        }
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Global);
  assert!(warnings.is_empty());
  // @keyframes foo
  assert_replace_dependency(input, &dependencies[0], "", ":local(");
  assert_local_class_dependency(input, &dependencies[1], ".class", true);
  assert_replace_dependency(input, &dependencies[2], "", ")");
  // @keyframes :local(bar)
  assert_replace_dependency(input, &dependencies[3], "", ":local(");
  assert_local_keyframes_decl_dependency(input, &dependencies[4], "bar");
  assert_replace_dependency(input, &dependencies[5], "", ")");
  assert_replace_dependency(input, &dependencies[6], "", ":local ");
  assert_local_class_dependency(input, &dependencies[7], ".class2", true);
  assert_local_keyframes_dependency(input, &dependencies[8], "bar");
  assert_eq!(dependencies.len(), 9);
}

#[test]
fn css_modules_container() {
  let input = indoc! {r#"
        .card {
            container-name: summary;
            container: card / inline-size;
        }
        @container summary (min-width: 400px) {
            .title {
                color: red;
            }
        }
        @container (width > 400px) {
            .content {
                color: blue;
            }
        }
        @container card style(--responsive: true) {
            .box {
                color: green;
            }
        }
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_local_class_dependency(input, &dependencies[0], ".card", false);
  assert_local_container_decl_dependency(input, &dependencies[1], "summary");
  assert_local_container_decl_dependency(input, &dependencies[2], "card");
  assert_local_container_dependency(input, &dependencies[3], "summary");
  assert_local_class_dependency(input, &dependencies[4], ".title", false);
  assert_local_class_dependency(input, &dependencies[5], ".content", false);
  assert_local_container_dependency(input, &dependencies[6], "card");
  assert_local_class_dependency(input, &dependencies[7], ".box", false);
  assert_eq!(dependencies.len(), 8);
}

#[test]
fn css_modules_function() {
  let input = indoc! {r#"
        @function --transparent(--color, --alpha) {
            result: var(--color);
        }
        .box {
            --base-color: #faa6ff;
            background-color: --transparent(var(--base-color), 0.8);
        }
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_local_function_decl_dependency(input, &dependencies[0], "transparent");
  assert_local_var_dependency(input, &dependencies[1], "color", None);
  assert_local_class_dependency(input, &dependencies[2], ".box", false);
  assert_local_var_decl_dependency(input, &dependencies[3], "base-color");
  assert_local_function_dependency(input, &dependencies[4], "transparent");
  assert_local_var_dependency(input, &dependencies[5], "base-color", None);
  assert_eq!(dependencies.len(), 6);
}

#[test]
fn css_modules_function_with_braced_argument() {
  let input = indoc! {r#"
        @function --max-plus-x(--list, --x) {
            result: calc(max(var(--list)) + var(--x));
        }
        .box {
            width: --max-plus-x({1px, 7px, 2px}, 3px);
        }
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_local_function_decl_dependency(input, &dependencies[0], "max-plus-x");
  assert_local_var_dependency(input, &dependencies[1], "list", None);
  assert_local_var_dependency(input, &dependencies[2], "x", None);
  assert_local_class_dependency(input, &dependencies[3], ".box", false);
  assert_local_function_dependency(input, &dependencies[4], "max-plus-x");
  assert_eq!(dependencies.len(), 5);
}

#[test]
fn css_modules_function_explicit_local() {
  let input = indoc! {r#"
        @function :local(--transparent-local)(--color, --alpha) {
            result: var(--color);
        }
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_replace_dependency(input, &dependencies[0], "", ":local(");
  assert_local_function_decl_dependency(input, &dependencies[1], "transparent-local");
  assert_replace_dependency(input, &dependencies[2], "", ")");
  assert_local_var_dependency(input, &dependencies[3], "color", None);
  assert_eq!(dependencies.len(), 4);
}

#[test]
fn css_modules_grid() {
  let input = indoc! {r#"
        .layout {
            grid-template-areas:
                "header header"
                "sidebar main";
            grid-area: header;
            grid-row: sidebar;
            grid-template: "hero hero" auto / 1fr 1fr;
        }
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_local_class_dependency(input, &dependencies[0], ".layout", false);
  assert_local_grid_decl_dependency(input, &dependencies[1], "header");
  assert_local_grid_decl_dependency(input, &dependencies[2], "header");
  assert_local_grid_decl_dependency(input, &dependencies[3], "sidebar");
  assert_local_grid_decl_dependency(input, &dependencies[4], "main");
  assert_local_grid_dependency(input, &dependencies[5], "header");
  assert_local_grid_dependency(input, &dependencies[6], "sidebar");
  assert_local_grid_decl_dependency(input, &dependencies[7], "hero");
  assert_local_grid_decl_dependency(input, &dependencies[8], "hero");
  assert_eq!(dependencies.len(), 9);
}

#[test]
fn css_modules_ident_start_includes_u0080() {
  let input = ".\u{80} {}";
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_local_class_dependency(input, &dependencies[0], ".\u{80}", false);
  assert_eq!(dependencies.len(), 1);
}

#[test]
fn css_modules_at_rule_1() {
  let input = indoc! {r#"
        @layer framework.container {
            .class {
                color: red;
            }
        }
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_local_class_dependency(input, &dependencies[0], ".class", false);
  assert_eq!(dependencies.len(), 1);
}

#[test]
fn css_modules_at_rule_2() {
  let input = indoc! {r#"
        @page {
            .class {
                color: red;
            }
        }
        @page :left, :top {
            .class2 {
                color: red;
            }
        }
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_local_class_dependency(input, &dependencies[0], ".class", false);
  assert_local_class_dependency(input, &dependencies[1], ".class2", false);
  assert_eq!(dependencies.len(), 2);
}

#[test]
fn css_modules_at_rule_3() {
  let input = indoc! {r#"
        .article-body {
            color: red;
        }
        @scope (.article-body) to (figure) {
            .img {
                background-color: goldenrod;
            }
        }
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_local_class_dependency(input, &dependencies[0], ".article-body", false);
  assert_local_class_dependency(input, &dependencies[1], ".article-body", false);
  assert_local_class_dependency(input, &dependencies[2], ".img", false);
  assert_eq!(dependencies.len(), 3);
}

#[test]
fn css_modules_composes_1() {
  let input = indoc! {r#"
        .exportName {
            composes: importName from "path/library.css", beforeName from global, importName secondImport from global, firstImport secondImport from "path/library.css";
            other: rule;
        }
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_local_class_dependency(input, &dependencies[0], ".exportName", false);
  assert_composes_dependency(
    input,
    &dependencies,
    &dependencies[1],
    "exportName",
    "importName",
    Some("\"path/library.css\""),
    "importName from \"path/library.css\"",
  );
  assert_composes_dependency(
    input,
    &dependencies,
    &dependencies[2],
    "exportName",
    "beforeName",
    Some("global"),
    "beforeName from global",
  );
  assert_composes_dependency(
    input,
    &dependencies,
    &dependencies[3],
    "exportName",
    "importName secondImport",
    Some("global"),
    "importName secondImport from global",
  );
  assert_composes_dependency(
    input,
    &dependencies,
    &dependencies[4],
    "exportName",
    "firstImport secondImport",
    Some("\"path/library.css\""),
    "firstImport secondImport from \"path/library.css\"",
  );
  assert_replace_dependency(
    input,
    &dependencies[5],
    "",
    r#"composes: importName from "path/library.css", beforeName from global, importName secondImport from global, firstImport secondImport from "path/library.css";"#,
  );
  assert_eq!(dependencies.len(), 6);
}

#[test]
fn css_modules_composes_2() {
  let input = indoc! {r#"
        .duplicate {
            composes: a from "./aa.css", b from "./bb.css", c from './cc.css', a from './aa.css', c from './cc.css'
        }
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_local_class_dependency(input, &dependencies[0], ".duplicate", false);
  assert_composes_dependency(
    input,
    &dependencies,
    &dependencies[1],
    "duplicate",
    "a",
    Some("\"./aa.css\""),
    "a from \"./aa.css\"",
  );
  assert_composes_dependency(
    input,
    &dependencies,
    &dependencies[2],
    "duplicate",
    "b",
    Some("\"./bb.css\""),
    "b from \"./bb.css\"",
  );
  assert_composes_dependency(
    input,
    &dependencies,
    &dependencies[3],
    "duplicate",
    "c",
    Some("'./cc.css'"),
    "c from './cc.css'",
  );
  assert_composes_dependency(
    input,
    &dependencies,
    &dependencies[4],
    "duplicate",
    "a",
    Some("'./aa.css'"),
    "a from './aa.css'",
  );
  assert_composes_dependency(
    input,
    &dependencies,
    &dependencies[5],
    "duplicate",
    "c",
    Some("'./cc.css'"),
    "c from './cc.css'",
  );
  assert_replace_dependency(
    input,
    &dependencies[6],
    "",
    r#"composes: a from "./aa.css", b from "./bb.css", c from './cc.css', a from './aa.css', c from './cc.css'"#,
  );
  assert_eq!(dependencies.len(), 7);
}

#[test]
fn css_modules_composes_3() {
  let input = indoc! {r#"
        .spaces {
            composes: importName importName2 from "path/library.css", importName3 importName4 from "path/library.css";
        }
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_local_class_dependency(input, &dependencies[0], ".spaces", false);
  assert_composes_dependency(
    input,
    &dependencies,
    &dependencies[1],
    "spaces",
    "importName importName2",
    Some("\"path/library.css\""),
    "importName importName2 from \"path/library.css\"",
  );
  assert_composes_dependency(
    input,
    &dependencies,
    &dependencies[2],
    "spaces",
    "importName3 importName4",
    Some("\"path/library.css\""),
    "importName3 importName4 from \"path/library.css\"",
  );
  assert_replace_dependency(
    input,
    &dependencies[3],
    "",
    r#"composes: importName importName2 from "path/library.css", importName3 importName4 from "path/library.css";"#,
  );
  assert_eq!(dependencies.len(), 4);
}

#[test]
fn css_modules_composes_4() {
  let input = indoc! {r#"
        .unknown {
            composes: foo bar, baz;
        }
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_local_class_dependency(input, &dependencies[0], ".unknown", false);
  assert_composes_dependency(
    input,
    &dependencies,
    &dependencies[1],
    "unknown",
    "foo bar",
    None,
    "foo bar",
  );
  assert_composes_dependency(
    input,
    &dependencies,
    &dependencies[2],
    "unknown",
    "baz",
    None,
    "baz",
  );
  assert_replace_dependency(input, &dependencies[3], "", r#"composes: foo bar, baz;"#);
  assert_eq!(dependencies.len(), 4);
}

#[test]
fn css_modules_composes_5() {
  let input = indoc! {r#"
        .mixed {
            composes: foo bar, baz, importName importName2 from "path/library.css"
        }
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_local_class_dependency(input, &dependencies[0], ".mixed", false);
  assert_composes_dependency(
    input,
    &dependencies,
    &dependencies[1],
    "mixed",
    "foo bar",
    None,
    "foo bar",
  );
  assert_composes_dependency(
    input,
    &dependencies,
    &dependencies[2],
    "mixed",
    "baz",
    None,
    "baz",
  );
  assert_composes_dependency(
    input,
    &dependencies,
    &dependencies[3],
    "mixed",
    "importName importName2",
    Some("\"path/library.css\""),
    "importName importName2 from \"path/library.css\"",
  );
  assert_replace_dependency(
    input,
    &dependencies[4],
    "",
    r#"composes: foo bar, baz, importName importName2 from "path/library.css""#,
  );
  assert_eq!(dependencies.len(), 5);
}

#[test]
fn css_modules_composes_6() {
  let input = indoc! {r#"
        .a, .b, .c {
            composes: foo
        }
        a, .b, .c {
            composes: foo
        }
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  // .a, .b, .c
  assert_local_class_dependency(input, &dependencies[0], ".a", false);
  assert_local_class_dependency(input, &dependencies[1], ".b", false);
  assert_local_class_dependency(input, &dependencies[2], ".c", false);
  assert_composes_dependency(
    input,
    &dependencies,
    &dependencies[3],
    "a b c",
    "foo",
    None,
    "foo",
  );
  assert_replace_dependency(input, &dependencies[4], "", "composes: foo");
  // a, .b, .c
  assert_warning(input, &warnings[0], "composes");
  assert_local_class_dependency(input, &dependencies[5], ".b", false);
  assert_local_class_dependency(input, &dependencies[6], ".c", false);
  assert_eq!(dependencies.len(), 7);
}

#[test]
fn css_modules_composes_from_does_not_leak_between_items() {
  let input = indoc! {r#"
        .exportName {
            composes: imported from "path/library.css", localName;
        }
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_local_class_dependency(input, &dependencies[0], ".exportName", false);
  assert_composes_dependency(
    input,
    &dependencies,
    &dependencies[1],
    "exportName",
    "imported",
    Some(r#""path/library.css""#),
    r#"imported from "path/library.css""#,
  );
  assert_composes_dependency(
    input,
    &dependencies,
    &dependencies[2],
    "exportName",
    "localName",
    None,
    "localName",
  );
  assert_replace_dependency(
    input,
    &dependencies[3],
    "",
    r#"composes: imported from "path/library.css", localName;"#,
  );
  assert_eq!(dependencies.len(), 4);
}

#[test]
fn css_modules_composes_7() {
  let input = indoc! {r#"
        .foo {
            .bar {}
            composes: global(a)
        }
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_local_class_dependency(input, &dependencies[0], ".foo", false);
  assert_local_class_dependency(input, &dependencies[1], ".bar", false);
  assert_composes_dependency(
    input,
    &dependencies,
    &dependencies[2],
    "foo",
    "a",
    Some("global"),
    "global(a)",
  );
  assert_replace_dependency(input, &dependencies[3], "", "composes: global(a)");
  assert_eq!(dependencies.len(), 4);
}

#[test]
fn css_modules_composes_8() {
  let input = indoc! {r#"
        .first,
        .second {
            color: green;
        }

        .base {
            background-color: red;
        }

        .third {
            composes: base;
        }
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_local_class_dependency(input, &dependencies[0], ".first", false);
  assert_local_class_dependency(input, &dependencies[1], ".second", false);
  assert_local_class_dependency(input, &dependencies[2], ".base", false);
  assert_local_class_dependency(input, &dependencies[3], ".third", false);
  assert_composes_dependency(
    input,
    &dependencies,
    &dependencies[4],
    "third",
    "base",
    None,
    "base",
  );
  assert_replace_dependency(input, &dependencies[5], "", "composes: base;");
  assert_eq!(dependencies.len(), 6);
}

#[test]
fn css_modules_composes_after_value_import_from_node_modules() {
  let input = indoc! {r#"
        @value color-grey from "./node_modules/@localpackage/color.css";

        .copyright {
          color: color-grey;
          composes: type-heading from "./node_modules/@localpackage/style.css";
          margin: 0;
          padding: 0;
        }
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_icss_import_from_dependency(
    input,
    &dependencies[0],
    "\"./node_modules/@localpackage/color.css\"",
  );
  assert_icss_import_value_dependency(input, &dependencies[1], "color-grey", "color-grey");
  assert_icss_export_value_dependency(input, &dependencies[2], "color-grey", "color-grey");
  assert_replace_dependency(
    input,
    &dependencies[3],
    "",
    r#"@value color-grey from "./node_modules/@localpackage/color.css";"#,
  );
  assert_local_class_dependency(input, &dependencies[4], ".copyright", false);
  assert_icss_symbol_dependency(input, &dependencies[5], "color-grey", "color-grey");
  assert_composes_dependency(
    input,
    &dependencies,
    &dependencies[6],
    "copyright",
    "type-heading",
    Some("\"./node_modules/@localpackage/style.css\""),
    r#"type-heading from "./node_modules/@localpackage/style.css""#,
  );
  assert_replace_dependency(
    input,
    &dependencies[7],
    "",
    r#"composes: type-heading from "./node_modules/@localpackage/style.css";"#,
  );
  assert_eq!(dependencies.len(), 8);
}

#[test]
fn icss_export_unexpected() {
  let input = ":export {\n/sl/ash;";
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert_warning(input, &warnings[0], ";");
  assert_eq!(warnings.len(), 1);
  assert_replace_dependency(input, &dependencies[0], "", ":export {\n/sl/ash");
  assert_eq!(dependencies.len(), 1);
}

#[test]
fn icss_import() {
  let input = indoc! {r#"
        :import(col.ors-2) {}
        :import("./colors.css") { i__blue: blue; i__red: red; }
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_icss_import_from_dependency(input, &dependencies[0], "col.ors-2");
  assert_replace_dependency(input, &dependencies[1], "", ":import(col.ors-2) {}");
  assert_icss_import_from_dependency(input, &dependencies[2], "\"./colors.css\"");
  assert_icss_import_value_dependency(input, &dependencies[3], "i__blue", "blue");
  assert_icss_import_value_dependency(input, &dependencies[4], "i__red", "red");
  assert_replace_dependency(
    input,
    &dependencies[5],
    "",
    ":import(\"./colors.css\") { i__blue: blue; i__red: red; }",
  );
  assert_eq!(dependencies.len(), 6);
}

#[test]
fn icss_import_with_comments_around_path() {
  let input = indoc! {r#"
        :import(   /* test */   "./export.modules.css"   /* test */   ) {
            IMPORTED_NAME: primary-color;
        }
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_icss_import_from_dependency(input, &dependencies[0], "\"./export.modules.css\"");
  assert_icss_import_value_dependency(input, &dependencies[1], "IMPORTED_NAME", "primary-color");
  assert_replace_dependency(
    input,
    &dependencies[2],
    "",
    indoc! {r#":import(   /* test */   "./export.modules.css"   /* test */   ) {
    IMPORTED_NAME: primary-color;
}"#},
  );
  assert_eq!(dependencies.len(), 3);
}

#[test]
fn icss_export() {
  let input = indoc! {r#"
        :export {
            a: a;
        }
        :export {
            abc: a b c;
            comments: abc/****/   /* hello world *//****/   def
        }
        :export{default:default}
        :export { $: abc; }
        :export { white space: a b c; }
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_icss_export_value_dependency(input, &dependencies[0], "a", "a");
  assert_replace_dependency(
    input,
    &dependencies[1],
    "",
    indoc! {r#":export {
            a: a;
        }"#},
  );
  assert_icss_export_value_dependency(input, &dependencies[2], "abc", "a b c");
  assert_icss_export_value_dependency(
    input,
    &dependencies[3],
    "comments",
    "abc/****/   /* hello world *//****/   def",
  );
  assert_replace_dependency(
    input,
    &dependencies[4],
    "",
    indoc! {r#":export {
            abc: a b c;
            comments: abc/****/   /* hello world *//****/   def
        }"#},
  );
  assert_icss_export_value_dependency(input, &dependencies[5], "default", "default");
  assert_replace_dependency(input, &dependencies[6], "", ":export{default:default}");
  assert_icss_export_value_dependency(input, &dependencies[7], "$", "abc");
  assert_replace_dependency(input, &dependencies[8], "", ":export { $: abc; }");
  assert_icss_export_value_dependency(input, &dependencies[9], "white space", "a b c");
  assert_replace_dependency(
    input,
    &dependencies[10],
    "",
    ":export { white space: a b c; }",
  );
  assert_eq!(dependencies.len(), 11);
}

#[test]
fn value_at_rule_export() {
  let input = indoc! {r#"
        @value primary: red;
        @value spacing calc(10px * 2);
        .button {
          color: primary;
          margin: spacing;
        }
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_icss_export_value_dependency(input, &dependencies[0], "primary", "red");
  assert_replace_dependency(input, &dependencies[1], "", "@value primary: red;");
  assert_icss_export_value_dependency(input, &dependencies[2], "spacing", "calc(10px * 2)");
  assert_replace_dependency(
    input,
    &dependencies[3],
    "",
    "@value spacing calc(10px * 2);",
  );
  assert_local_class_dependency(input, &dependencies[4], ".button", false);
  assert_icss_symbol_dependency(input, &dependencies[5], "primary", "primary");
  assert_icss_symbol_dependency(input, &dependencies[6], "spacing", "spacing");
  assert_eq!(dependencies.len(), 7);
}

#[test]
fn value_at_rule_missing_value_warns_and_keeps_dependencies() {
  let input = "@value test;";
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert_eq!(
    warnings,
    [Warning::new(
      Range::new(0, 12),
      WarningKind::Unexpected {
        message: "Broken '@value' at-rule",
      },
    )]
  );
  assert_icss_export_value_dependency(input, &dependencies[0], "test", "");
  assert_replace_dependency(input, &dependencies[1], "", input);
  assert_eq!(dependencies.len(), 2);
}

#[test]
fn rspack_value_at_rule_only_bare_name_is_newly_broken() {
  let input = r#"@value v-comment-broken:;
@value v-comment-broken-v1:/* comment */;
@value v-empty: ;
@value v-empty-v2:   ;
@value v-empty-v3: /* comment */;
@value multiline-empty: /*
    multiline
    comment
*/;
@value;
@value test;"#;
  let (_, warnings) = collect_dependencies(input, Mode::Local);
  assert_eq!(
    warnings
      .iter()
      .map(|warning| Lexer::slice_range(input, warning.range()).unwrap())
      .collect::<Vec<_>>(),
    ["@value;", "@value test;"]
  );
}

#[test]
fn value_at_rule_comment_separated_from_has_clean_request() {
  let quoted = r#"@value/* test */test-v2/* test */from/* test */"./colors.module.css"/* test */;"#;
  let (dependencies, warnings) = collect_dependencies(quoted, Mode::Local);
  assert!(warnings.is_empty());
  assert_icss_import_from_dependency(quoted, &dependencies[0], r#""./colors.module.css""#);

  let identifier = "@value/* test */red/* test */from/* test */colors/* test */;";
  let (dependencies, warnings) = collect_dependencies(identifier, Mode::Local);
  assert!(warnings.is_empty());
  assert_icss_import_from_dependency(identifier, &dependencies[0], "colors");
}

#[test]
fn value_at_rule_preserves_unicode_byte_offsets() {
  let declaration = "@value café: red;";
  let (dependencies, warnings) = collect_dependencies(declaration, Mode::Local);
  assert!(warnings.is_empty());
  assert_icss_export_value_dependency(declaration, &dependencies[0], "café", "red");
  assert_eq!(dependencies.len(), 2);

  let import = r#"@value café from "./café.module.css";"#;
  let (dependencies, warnings) = collect_dependencies(import, Mode::Local);
  assert!(warnings.is_empty());
  assert_icss_import_from_dependency(import, &dependencies[0], r#""./café.module.css""#);
  assert_icss_import_value_dependency(import, &dependencies[1], "café", "café");
  assert_icss_export_value_dependency(import, &dependencies[2], "café", "café");
  assert_eq!(dependencies.len(), 4);
}

#[test]
fn value_at_rule_import_path_handles_escaped_quote() {
  let input = r#"@value color from "./a\"b.css";"#;
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_icss_import_from_dependency(input, &dependencies[0], r#""./a\"b.css""#);
  assert_icss_import_value_dependency(input, &dependencies[1], "color", "color");
  assert_icss_export_value_dependency(input, &dependencies[2], "color", "color");
  assert_eq!(dependencies.len(), 4);
}

#[test]
fn rspack_comment_heavy_value_declarations_are_parsed() {
  let cases = [
    (
      "@value/* test */blue-v5/* test */:/* test */red/* test */;",
      "blue-v5",
      "/* test */red/* test */",
    ),
    (
      "@value/* test */blue-v6/* test *//* test */red/* test */;",
      "blue-v6",
      "/* test *//* test */red/* test */",
    ),
    (
      "@value   /* test */   coolShadow-v4   /* test */   0 11px 15px -7px rgba(0,0,0,.2),0 24px 38px 3px rgba(0,0,0,.14)   ;",
      "coolShadow-v4",
      "/* test */   0 11px 15px -7px rgba(0,0,0,.2),0 24px 38px 3px rgba(0,0,0,.14)",
    ),
    (
      "@value/* test */coolShadow-v5/* test */0 11px 15px -7px rgba(0,0,0,.2),0 24px 38px 3px rgba(0,0,0,.14);",
      "coolShadow-v5",
      "/* test */0 11px 15px -7px rgba(0,0,0,.2),0 24px 38px 3px rgba(0,0,0,.14)",
    ),
  ];

  for (input, target, value) in cases {
    let (dependencies, _) = collect_dependencies(input, Mode::Local);
    let export = dependencies
      .iter()
      .find(|dependency| {
        matches!(
            dependency,
            Dependency::ICSSExportValue { prop, .. } if *prop == target
        )
      })
      .expect("comment-heavy @value declaration should be parsed");
    assert_icss_export_value_dependency(input, export, target, value);
  }
}

#[test]
fn css_mode_data_queries_are_not_local() {
  let mode_data = ModeData::new(Mode::Css);
  assert!(!mode_data.is_current_local_mode());
  assert!(!mode_data.is_property_local_mode());
}

#[test]
fn value_at_rule_preserves_comment_only_value() {
  let input = "@value v-empty: /* comment */;";
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_icss_export_value_dependency(input, &dependencies[0], "v-empty", " /* comment */");
}

#[test]
fn rspack_comment_heavy_value_imports_are_parsed() {
  let cases = [
    (
      "@value /* test */ test-v1 /* test */ from /* test */ \"./colors.module.css\" /* test */;",
      "test-v1",
      "test-v1",
    ),
    (
      "@value/* test */test-v2/* test */from/* test */\"./colors.module.css\"/* test */;",
      "test-v2",
      "test-v2",
    ),
    (
      "@value/* test */(/* test */blue-v1/* test */as/* test */my-name-q/* test */)/* test */from/* test */\"./colors.module.css\"/* test */;",
      "my-name-q",
      "blue-v1",
    ),
    (
      "@value/* test */danger/* test */:/* test */error/* test */from/* test */\"./colors.module.css\"/* test */;",
      "danger",
      "error",
    ),
    (
      "@value /*
    multiline comment
*/ red-v3 /*
    multiline comment
*/ from /*
    multiline comment
*/ \"./colors.module.css\" /*
    multiline comment
*/;",
      "red-v3",
      "red-v3",
    ),
  ];

  for (input, local_name, import_name) in cases {
    let (dependencies, _) = collect_dependencies(input, Mode::Local);
    let import_from = dependencies
      .iter()
      .find(|dependency| matches!(dependency, Dependency::ICSSImportFrom { .. }))
      .expect("comment-heavy import should retain its import-from dependency");
    assert_icss_import_from_dependency(input, import_from, r#""./colors.module.css""#);
    let import_value = dependencies
      .iter()
      .find(|dependency| {
        matches!(
            dependency,
            Dependency::ICSSImportValue { prop, .. } if *prop == local_name
        )
      })
      .expect("comment-heavy @value import item should be parsed");
    assert_icss_import_value_dependency(input, import_value, local_name, import_name);
  }
}

#[test]
fn value_at_rule_import() {
  let input = indoc! {r#"
        @value red from "./colors.css";
        @value blue as sky, green from "./palette.css";
        @value (black as ink, white) from "./theme.css";
        .button {
          color: red;
          background: sky;
          border-color: green;
          outline-color: ink;
          text-decoration-color: white;
        }
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  assert_icss_import_from_dependency(input, &dependencies[0], "\"./colors.css\"");
  assert_icss_import_value_dependency(input, &dependencies[1], "red", "red");
  assert_icss_export_value_dependency(input, &dependencies[2], "red", "red");
  assert_replace_dependency(
    input,
    &dependencies[3],
    "",
    "@value red from \"./colors.css\";",
  );

  assert_icss_import_from_dependency(input, &dependencies[4], "\"./palette.css\"");
  assert_icss_import_value_dependency(input, &dependencies[5], "sky", "blue");
  assert_icss_export_value_dependency(input, &dependencies[6], "sky", "sky");
  assert_icss_import_value_dependency(input, &dependencies[7], "green", "green");
  assert_icss_export_value_dependency(input, &dependencies[8], "green", "green");
  assert_replace_dependency(
    input,
    &dependencies[9],
    "",
    "@value blue as sky, green from \"./palette.css\";",
  );

  assert_icss_import_from_dependency(input, &dependencies[10], "\"./theme.css\"");
  assert_icss_import_value_dependency(input, &dependencies[11], "ink", "black");
  assert_icss_export_value_dependency(input, &dependencies[12], "ink", "ink");
  assert_icss_import_value_dependency(input, &dependencies[13], "white", "white");
  assert_icss_export_value_dependency(input, &dependencies[14], "white", "white");
  assert_replace_dependency(
    input,
    &dependencies[15],
    "",
    "@value (black as ink, white) from \"./theme.css\";",
  );
  assert_local_class_dependency(input, &dependencies[16], ".button", false);
  assert_icss_symbol_dependency(input, &dependencies[17], "red", "red");
  assert_icss_symbol_dependency(input, &dependencies[18], "sky", "sky");
  assert_icss_symbol_dependency(input, &dependencies[19], "green", "green");
  assert_icss_symbol_dependency(input, &dependencies[20], "ink", "ink");
  assert_icss_symbol_dependency(input, &dependencies[21], "white", "white");
  let value_import_items = dependencies
    .value_at_rule_import_items()
    .iter()
    .map(|item| (item.local_name(), item.import_name()))
    .collect::<Vec<_>>();
  assert_eq!(
    value_import_items,
    [
      ("red", "red"),
      ("sky", "blue"),
      ("green", "green"),
      ("ink", "black"),
      ("white", "white"),
    ]
  );
  assert_eq!(dependencies.len(), 22);
}

#[test]
fn weird_composes() {
  let input = indoc! {r#"
        .from { color: red; }
        :local(.exportName31) {
	        composes: from from;
        }
    "#};
  let (dependencies, warnings) = collect_dependencies(input, Mode::Global);
  assert!(warnings.is_empty());
  assert_local_class_dependency(input, &dependencies[1], ".exportName31", true);
  assert_composes_dependency(
    input,
    &dependencies,
    &dependencies[3],
    "exportName31",
    "from from",
    None,
    "from from",
  );
}
