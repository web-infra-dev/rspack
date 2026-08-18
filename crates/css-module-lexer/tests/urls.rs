use crate::support::*;

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
    .expect("test setup must produce the expected value");
  let Dependency::LocalKeyframesDecl { name, range } = declaration else {
    unreachable!();
  };
  assert_eq!(*name, r"sl\69 de");
  assert_eq!(Lexer::slice_range(input, range), Some(r"sl\69 de"));
  let usage = dependencies
    .iter()
    .find(|dependency| matches!(dependency, Dependency::LocalKeyframes { .. }))
    .expect("test setup must produce the expected value");
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
    .expect("test setup must produce the expected value");
  let Dependency::LocalVarDecl { name, range } = declaration else {
    unreachable!();
  };
  assert_eq!(*name, r"v\61 r");
  assert_eq!(Lexer::slice_range(input, range), Some(r"--v\61 r"));
  let usage = dependencies
    .iter()
    .find(|dependency| matches!(dependency, Dependency::LocalVar { .. }))
    .expect("test setup must produce the expected value");
  let Dependency::LocalVar {
    name, from, range, ..
  } = usage
  else {
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
    .expect("test setup must produce the expected value");
  let Dependency::Composes {
    local_classes,
    names,
    from,
    range,
    ..
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
    .expect("test setup must produce the expected value");
  assert_icss_symbol_dependency(input, symbol, r"l\6f cal", r"l\6f cal");

  let input = r":export { f\6f o: red; } .x { color: f\6f o; }";
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);
  assert!(warnings.is_empty());
  let export = dependencies
    .iter()
    .find(|dependency| matches!(dependency, Dependency::ICSSExportValue { .. }))
    .expect("test setup must produce the expected value");
  assert_icss_export_value_dependency(input, export, r"f\6f o", "red");
  let symbol = dependencies
    .iter()
    .find(|dependency| matches!(dependency, Dependency::ICSSSymbol { .. }))
    .expect("test setup must produce the expected value");
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
