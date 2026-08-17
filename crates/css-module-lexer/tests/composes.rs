use crate::{scope::*, support::*};

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
fn css_modules_composes_distinguishes_global_keyword_from_requests() {
  let input = indoc! {r#"
        .exportName {
            composes: lower from global, upper from GLOBAL, escaped from g\6c obal, quoted from "global";
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
    "lower",
    Some("global"),
    "lower from global",
  );
  assert_composes_dependency(
    input,
    &dependencies,
    &dependencies[2],
    "exportName",
    "upper",
    Some("GLOBAL"),
    "upper from GLOBAL",
  );
  assert_composes_dependency(
    input,
    &dependencies,
    &dependencies[3],
    "exportName",
    "escaped",
    Some(r"g\6c obal"),
    r"escaped from g\6c obal",
  );
  let Dependency::Composes {
    from,
    from_is_global,
    ..
  } = &dependencies[4]
  else {
    panic!("unexpected dependency");
  };
  assert_eq!(*from, Some(r#""global""#));
  assert!(!*from_is_global);
  assert_composes_dependency(
    input,
    &dependencies,
    &dependencies[4],
    "exportName",
    "quoted",
    Some(r#""global""#),
    r#"quoted from "global""#,
  );
  for dependency in &dependencies.dependencies()[1..4] {
    assert!(matches!(
      dependency,
      Dependency::Composes {
        from_is_global: true,
        ..
      }
    ));
  }
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
fn export_nested_class() {
  test(
    indoc! {r#"
            :local(.exportName):not(:local(.otherExportName).global) {
                color: green;
            }

            :local(.exportName):has(:local(.otherExportName), :local(.otherExportName2)) {
                color: red;
            }
        "#},
    indoc! {r#"
            ._input__exportName:not(._input__otherExportName.global) {
                color: green;
            }

            ._input__exportName:has(._input__otherExportName, ._input__otherExportName2) {
                color: red;
            }

            :export {
                exportName: _input__exportName;
                otherExportName: _input__otherExportName;
                otherExportName2: _input__otherExportName2;
            }
        "#},
  );
}

#[test]
fn export_with_composes() {
  test(
    indoc! {r#"
            :local(.otherClass) { background: red; } :local(.exportName) { compose-with: otherClass; color: green; }
        "#},
    indoc! {r#"
            ._input__otherClass { background: red; } ._input__exportName {  color: green; }

            :export {
                otherClass: _input__otherClass;
                exportName: _input__exportName _input__otherClass;
            }
        "#},
  );
}

#[test]
fn export_with_composes_imported_class() {
  // TODO: replace import value, should be `exportName: _lib_extender__exportName imported_otherClass;`
  test(
    indoc! {r#"
            :import("./file.css") {
                imported_otherClass: otherClass;
            }
            :local(.exportName) {
                composes: imported_otherClass;
                color: green;
            }
        "#},
    indoc! {r#"
            :import("./file.css") {
                imported_otherClass: otherClass;
            }
            ._input__exportName {
                __CSS_MODULE_LEXER_WHITESPACE__
                color: green;
            }

            :export {
                exportName: _input__exportName _input__imported_otherClass;
            }
        "#},
  );
}

#[test]
fn export_with_global_composes() {
  test(
    indoc! {r#"
            .otherClass { background: red; }
            .andAgain { font-size: 2em; }
            .aThirdClass { color: red; }
            :local(.exportName) { compose-with: global(otherClass) global(andAgain); compose-with: global(aThirdClass); color: green; }
        "#},
    indoc! {r#"
            .otherClass { background: red; }
            .andAgain { font-size: 2em; }
            .aThirdClass { color: red; }
            ._input__exportName {   color: green; }

            :export {
                exportName: _input__exportName otherClass andAgain aThirdClass;
            }
        "#},
  );
}

#[test]
fn export_with_multiple_composes() {
  test(
    indoc! {r#"
            :local(.otherClass) { background: red; }
            :local(.andAgain) { font-size: 2em; }
            :local(.aThirdClass) { color: red; }
            :local(.exportName) { compose-with: otherClass andAgain; compose-with: aThirdClass; color: green; }
        "#},
    indoc! {r#"
            ._input__otherClass { background: red; }
            ._input__andAgain { font-size: 2em; }
            ._input__aThirdClass { color: red; }
            ._input__exportName {   color: green; }

            :export {
                otherClass: _input__otherClass;
                andAgain: _input__andAgain;
                aThirdClass: _input__aThirdClass;
                exportName: _input__exportName _input__otherClass _input__andAgain _input__aThirdClass;
            }
        "#},
  );
}

#[test]
fn export_with_transitive_composes() {
  test(
    indoc! {r#"
            :local(.aThirdClass) {
                font-size: 2em;
            }
            :local(.otherClass) {
                composes: aThirdClass;
                background: red;
            }
            :local(.exportName) {
                composes: otherClass;
                color: green;
            }
        "#},
    indoc! {r#"
            ._input__aThirdClass {
                font-size: 2em;
            }
            ._input__otherClass {
                __CSS_MODULE_LEXER_WHITESPACE__
                background: red;
            }
            ._input__exportName {
                __CSS_MODULE_LEXER_WHITESPACE__
                color: green;
            }

            :export {
                aThirdClass: _input__aThirdClass;
                otherClass: _input__otherClass _input__aThirdClass;
                exportName: _input__exportName _input__otherClass _input__aThirdClass;
            }
        "#},
  );
}

#[test]
fn ignore_custom_property_set() {
  test(
    indoc! {r#"
            :root {
                --title-align: center;
                --sr-only: {
                    position: absolute;
                }
            }
        "#},
    indoc! {r#"
            :root {
                --title-align: center;
                --sr-only: {
                    position: absolute;
                }
            }
        "#},
  );
}

#[test]
fn multiple_composes() {
  // TODO: replace import value
  test(
    indoc! {r#"
            :import("path") {
                i__i_a_0: a;
                i__i_b_0: b;
                i__i_c_0: c;
                i__i_d_0: d;
            }
            :local(.class) {
                composes: i__i_a_0 i__i_b_0, i__i_c_0, global(d) global(e), global(f), i__i_d_0;
                color: red;
            }
        "#},
    indoc! {r#"
            :import("path") {
                i__i_a_0: a;
                i__i_b_0: b;
                i__i_c_0: c;
                i__i_d_0: d;
            }
            ._input__class {
                __CSS_MODULE_LEXER_WHITESPACE__
                color: red;
            }

            :export {
                class: _input__class _input__i__i_a_0 _input__i__i_b_0 _input__i__i_c_0 d e f _input__i__i_d_0;
            }
        "#},
  );
}

#[test]
fn nested_rule() {
  test(
    indoc! {r#"
            :root {
                --test: {
                    --test: foo;
                    --bar: 1;
                }
            }
        "#},
    indoc! {r#"
            :root {
                --test: {
                    --test: foo;
                    --bar: 1;
                }
            }
        "#},
  );
}

#[test]
fn nothing() {
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
        "#},
  );
}

#[test]
fn options_generate_scoped_name() {
  test(
    indoc! {r#"
            :local(.exportName) {
                color: green;
            }

            :local(.exportName):hover {
                color: red;
            }
        "#},
    indoc! {r#"
            ._input__exportName {
                color: green;
            }

            ._input__exportName:hover {
                color: red;
            }

            :export {
                exportName: _input__exportName;
            }
        "#},
  );
}
