use crate::{scope::*, support::*};

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
      .map(|warning| Lexer::slice_range(input, warning.range())
        .expect("test setup must produce the expected value"))
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
#[test]
fn escape_sequence() {
  test(
    indoc! {r#"
            :local(.smile) {
                color: red;
            }

            :local(.smile) :local(.smile) {
                color: red;
            }

            :local(.smile) :local(.smile) :local(.smile) {
                color: red;
            }

            :local(.smile_with_A) {
                color: red;
            }

            .\1F600  :local(.smile) {
                color: red;
            }

            :local(.smile) .\1F600 {
                color: red;
            }

            .\1F600  :local(.smile) .\1F600 {
                color: red;
            }

            .\1F600  :local(.smile_with_A) .\1F600 {
                color: red;
            }

            #\1F600  :local(#smile) #\1F600 {
                color: red;
            }

            #\1F600  :local(#smile_with_A) #\1F600 {
                color: red;
            }

            .a :local(.smile) b {
                color: red;
            }

            :local(.smile) > :local(.smile) > :local(.smile) {
                color: red;
            }

            .\1F600 :local(.smile) {
                color: red;
            }

            .\1F600:local(.smile) {
                color: red;
            }

            .\1F600  :local(.smile) {
                color: red;
            }

            :local(.smile) .a {
                color: red;
            }

            :local(.smile).a {
                color: red;
            }

            .a :local(.smile) {
                color: red;
            }

            .a:local(.smile) {
                color: red;
            }
        "#},
    indoc! {r#"
            ._input__smile {
                color: red;
            }

            ._input__smile ._input__smile {
                color: red;
            }

            ._input__smile ._input__smile ._input__smile {
                color: red;
            }

            ._input__smile_with_A {
                color: red;
            }

            .\1F600  ._input__smile {
                color: red;
            }

            ._input__smile .\1F600 {
                color: red;
            }

            .\1F600  ._input__smile .\1F600 {
                color: red;
            }

            .\1F600  ._input__smile_with_A .\1F600 {
                color: red;
            }

            #\1F600  #_input__smile #\1F600 {
                color: red;
            }

            #\1F600  #_input__smile_with_A #\1F600 {
                color: red;
            }

            .a ._input__smile b {
                color: red;
            }

            ._input__smile > ._input__smile > ._input__smile {
                color: red;
            }

            .\1F600 ._input__smile {
                color: red;
            }

            .\1F600._input__smile {
                color: red;
            }

            .\1F600  ._input__smile {
                color: red;
            }

            ._input__smile .a {
                color: red;
            }

            ._input__smile.a {
                color: red;
            }

            .a ._input__smile {
                color: red;
            }

            .a._input__smile {
                color: red;
            }

            :export {
                smile_with_A: _input__smile_with_A;
                smile: _input__smile;
            }
        "#},
  );
}

#[test]
fn export_child_class() {
  test(
    indoc! {r#"
            :local(.simple) {
                color: red;
            }

            :local(.simple) h1 {
                color: blue;
            }
        "#},
    indoc! {r#"
            ._input__simple {
                color: red;
            }

            ._input__simple h1 {
                color: blue;
            }

            :export {
                simple: _input__simple;
            }
        "#},
  );
}

#[test]
fn export_class_attribute() {
  // should be ._input__exportName2[class=_input__exportName1]
  // but in css-loader after transformed by local_by_default
  // :local(.exportName2[class="exportName1"]) will become :local(.exportName2)[class="exportName1"]
  // so the result of css-loader is same with us
  test(
    indoc! {r#"
            :local(.exportName1) {
                color: red;
            }

            :local(.exportName2) {
                color: green;
            }

            :local(.exportName2[class="exportName1"]) {
                color: blue;
            }
        "#},
    indoc! {r#"
            ._input__exportName1 {
                color: red;
            }

            ._input__exportName2 {
                color: green;
            }

            ._input__exportName2[class="exportName1"] {
                color: blue;
            }

            :export {
                exportName1: _input__exportName1;
                exportName2: _input__exportName2;
            }
        "#},
  );
}

#[test]
fn export_class_path() {
  test(
    indoc! {r#"
            :local(.exportName) {
                color: green;
            }
        "#},
    indoc! {r#"
            ._input__exportName {
                color: green;
            }

            :export {
                exportName: _input__exportName;
            }
        "#},
  );
}

#[test]
fn export_difficult() {
  // should rename :local() in animation, but using :local() in property is not valid css
  test(
    indoc! {r#"
            @keyframes :local(fade-in) {
                from {
                    opacity: 0;
                }
            }
            __CSS_MODULE_LEXER_WHITESPACE__
            @-webkit-keyframes :local(fade-out) {
                to {
                    opacity: 0;
                }
            }
            __CSS_MODULE_LEXER_WHITESPACE__
            :local(.fadeIn) {
                animation: :local(fade-in) 5s,
                :local(fade-out) 1s :local(wrong);
                content: :local(fade-in), wrong, "difficult, :local(wrong)" :local(wrong);
            }
        "#},
    indoc! {r#"
            @keyframes _input__fade-in {
                from {
                    opacity: 0;
                }
            }

            @-webkit-keyframes _input__fade-out {
                to {
                    opacity: 0;
                }
            }

            ._input__fadeIn {
                animation: :local(fade-in) 5s,
                :local(fade-out) 1s :local(wrong);
                content: :local(fade-in), wrong, "difficult, :local(wrong)" :local(wrong);
            }

            :export {
                fade-in: _input__fade-in;
                fade-out: _input__fade-out;
                fadeIn: _input__fadeIn;
            }
        "#},
  );
}

#[test]
fn export_global_class() {
  // we don't support exportGlobals
  test(
    indoc! {r#"
            .exportName {
                color: green;
            }

            .exportName:hover {
                color: red;
            }

            @media screen {
                body {
                    background: red;
                }
            }

            :local(.testLocal) {
                color: blue;
            }
        "#},
    indoc! {r#"
            .exportName {
                color: green;
            }

            .exportName:hover {
                color: red;
            }

            @media screen {
                body {
                    background: red;
                }
            }

            ._input__testLocal {
                color: blue;
            }

            :export {
                testLocal: _input__testLocal;
            }
        "#},
  );
}

#[test]
fn export_global_id() {
  test(
    indoc! {r#"
            #exportName {
                color: green;
            }

            #exportName:hover {
                color: red;
            }

            @media screen {
                #exportName-2 {
                    background: red;
                }
            }

            :local(#exportName-3) {
                color: green;
            }
        "#},
    indoc! {r#"
            #exportName {
                color: green;
            }

            #exportName:hover {
                color: red;
            }

            @media screen {
                #exportName-2 {
                    background: red;
                }
            }

            #_input__exportName-3 {
                color: green;
            }

            :export {
                exportName-3: _input__exportName-3;
            }
        "#},
  );
}

#[test]
fn export_keyframes() {
  test(
    indoc! {r#"
            @keyframes :local(fade-in) {
                from {
                    opacity: 0;
                }
                100% {
                    opacity: 1;
                }
            }

            @keyframes fade {
                from {
                    opacity: 0.5;
                }
            }

            :local(.fadeIn) {
                animation-name: :local(fade-in);
            }

            :local(.fadeIn) {
                animation: 2s :local(fade-in);
            }

            :local(.fadeIn) {
                animation: :local(fade-in) 2s;
            }
        "#},
    indoc! {r#"
            @keyframes _input__fade-in {
                from {
                    opacity: 0;
                }
                100% {
                    opacity: 1;
                }
            }

            @keyframes fade {
                from {
                    opacity: 0.5;
                }
            }

            ._input__fadeIn {
                animation-name: :local(fade-in);
            }

            ._input__fadeIn {
                animation: 2s :local(fade-in);
            }

            ._input__fadeIn {
                animation: :local(fade-in) 2s;
            }

            :export {
                fade-in: _input__fade-in;
                fadeIn: _input__fadeIn;
            }
        "#},
  );
}

#[test]
fn export_keywords_selector() {
  test(
    indoc! {r#"
            :local(.constructor) {
                color: green;
            }

            :local(.toString) {
                color: red;
            }
        "#},
    indoc! {r#"
            ._input__constructor {
                color: green;
            }
            __CSS_MODULE_LEXER_WHITESPACE__
            ._input__toString {
                color: red;
            }
            __CSS_MODULE_LEXER_WHITESPACE__
            :export {
                constructor: _input__constructor;
                toString: _input__toString;
            }
        "#},
  );
}

#[test]
fn export_multiple_classes() {
  test(
    indoc! {r#"
            :local(.exportName) :local(.otherExport) {
                color: green;
            }

            :local(.exportName):local(.otherExport) {
                color: red;
            }
        "#},
    indoc! {r#"
            ._input__exportName ._input__otherExport {
                color: green;
            }

            ._input__exportName._input__otherExport {
                color: red;
            }

            :export {
                exportName: _input__exportName;
                otherExport: _input__otherExport;
            }
        "#},
  );
}
