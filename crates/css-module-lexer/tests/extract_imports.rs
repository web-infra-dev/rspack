use std::{
  collections::{HashMap, HashSet},
  fmt::Write,
};

use css_module_lexer::{
  Dependency, Lexer, Mode, Range, Warning, WarningKind, collect_dependencies,
};
use indoc::indoc;
use linked_hash_map::LinkedHashMap;

#[derive(Debug, Default, Clone, Hash, PartialEq, Eq)]
pub struct ExtractImports;

enum StateMarker {
  Permanent,
  Temporary,
}

fn serialize_identifier(mut value: &str) -> String {
  fn serialize_name(value: &str, output: &mut String) {
    for character in value.chars() {
      match character {
        '\0' => output.push('\u{FFFD}'),
        '0'..='9' | 'A'..='Z' | 'a'..='z' | '_' | '-' => output.push(character),
        character if !character.is_ascii() => output.push(character),
        character if character.is_ascii_control() => {
          write!(output, "\\{:x} ", character as u32)
            .expect("test setup must produce the expected value");
        }
        character => {
          output.push('\\');
          output.push(character);
        }
      }
    }
  }

  let mut output = String::new();
  if let Some(rest) = value.strip_prefix("--") {
    output.push_str("--");
    serialize_name(rest, &mut output);
    return output;
  }
  if value == "-" {
    return "\\-".to_string();
  }
  if let Some(rest) = value.strip_prefix('-') {
    output.push('-');
    value = rest;
  }
  if let Some(digit @ '0'..='9') = value.chars().next() {
    write!(output, "\\{:x} ", digit as u32).expect("test setup must produce the expected value");
    value = &value[digit.len_utf8()..];
  }
  serialize_name(value, &mut output);
  output
}

fn add_import_to_graph(
  import: &str,
  rule_index: u32,
  graph: &mut LinkedHashMap<String, Vec<String>>,
  visited: &mut HashSet<String>,
  siblings: &mut HashMap<u32, Vec<String>>,
) {
  let visited_id = format!("{rule_index}_{import}");
  if visited.contains(&visited_id) {
    return;
  }
  let children = graph.entry(import.to_string()).or_default();
  if let Some(siblings) = siblings.get(&rule_index) {
    children.extend(siblings.iter().cloned());
  }
  visited.insert(visited_id);
  siblings
    .entry(rule_index)
    .or_default()
    .push(import.to_string());
}

fn walk_graph(
  import: &str,
  graph: &LinkedHashMap<String, Vec<String>>,
  state: &mut HashMap<String, StateMarker>,
  result: &mut Vec<String>,
  warnings: &mut Vec<Warning<'_>>,
) {
  if let Some(marker) = state.get(import) {
    match marker {
      StateMarker::Permanent => {
        return;
      }
      StateMarker::Temporary => {
        warnings.push(Warning::new(
          Range::new(0, 0),
          WarningKind::Unexpected {
            message: "Failed to resolve order of composed modules",
          },
        ));
        return;
      }
    }
  }
  state.insert(import.to_string(), StateMarker::Temporary);
  for child in &graph[import] {
    walk_graph(child, graph, state, result, warnings);
  }
  state.insert(import.to_string(), StateMarker::Permanent);
  result.push(import.to_string());
}

pub(crate) fn topological_sort(
  graph: &LinkedHashMap<String, Vec<String>>,
  warnings: &mut Vec<Warning<'_>>,
) -> Vec<String> {
  let mut result = Vec::new();
  let mut state = HashMap::new();
  for import in graph.keys() {
    walk_graph(import, graph, &mut state, &mut result, warnings);
  }
  result
}

impl ExtractImports {
  pub fn transform<'s>(&self, input: &'s str) -> (String, Vec<Warning<'s>>) {
    let mut imported = String::new();
    let mut result = String::new();
    let mut index = 0;
    let mut composes_contents = Vec::new();
    let mut postfix = 0;
    let mut imports: LinkedHashMap<String, LinkedHashMap<String, String>> = LinkedHashMap::new();
    let mut rule_index = 0;
    let mut graph: LinkedHashMap<String, Vec<String>> = LinkedHashMap::new();
    let mut visited = HashSet::new();
    let mut siblings = HashMap::new();
    let (dependencies, mut warnings) = collect_dependencies(input, Mode::Local);
    for dependency in &dependencies {
      match dependency {
        Dependency::LocalClass { .. } | Dependency::LocalId { .. } => {
          rule_index += 1;
        }
        Dependency::Composes {
          names,
          from,
          from_is_global,
          ..
        } => {
          let names = dependencies.composes_names(*names);
          let mut composes_content = String::new();
          if let Some(from) = from {
            if *from_is_global {
              for i in 0..names.len() {
                let name = names[i];
                composes_content += "global(";
                composes_content += &serialize_identifier(name);
                composes_content += ")";
                if i + 1 != names.len() {
                  composes_content += " ";
                }
              }
            } else {
              let path = from.trim_matches(|c| c == '\'' || c == '"');
              add_import_to_graph(path, rule_index, &mut graph, &mut visited, &mut siblings);
              let values = imports.entry(path.to_string()).or_default();
              for i in 0..names.len() {
                let name = names[i];
                if let Some(value) = values.get(name) {
                  composes_content += value;
                } else {
                  let serialized_name = name
                    .chars()
                    .map(|character| {
                      if character.is_ascii_alphanumeric() || character == '_' {
                        character
                      } else {
                        '_'
                      }
                    })
                    .collect::<String>();
                  let value = format!("i__imported_{serialized_name}_{postfix}");
                  postfix += 1;
                  composes_content += value.as_ref();
                  values.insert(name.to_string(), value);
                }
                if i + 1 != names.len() {
                  composes_content += " ";
                }
              }
            }
          } else {
            for i in 0..names.len() {
              let name = names[i];
              composes_content += &serialize_identifier(name);
              if i + 1 != names.len() {
                composes_content += " ";
              }
            }
          }
          composes_contents.push(composes_content);
        }
        Dependency::Replace { content, range } => {
          if !composes_contents.is_empty() {
            let composes_contents = std::mem::take(&mut composes_contents);
            result += Lexer::slice_range(input, &Range::new(index, range.start))
              .expect("test setup must produce the expected value");
            result += "composes: ";
            result += &composes_contents.join(", ");
            result += ";";
            index = range.end;
          } else {
            let original =
              Lexer::slice_range(input, range).expect("test setup must produce the expected value");
            if original.starts_with(":export") || original.starts_with(":import(") {
              result += Lexer::slice_range(input, &Range::new(index, range.start))
                .expect("test setup must produce the expected value");
              result += content;
              index = range.end;
            }
          }
        }
        Dependency::ICSSImportFrom { path } => {
          let path = path.trim_matches(|c| c == '\'' || c == '"');
          imports.insert(path.to_string(), LinkedHashMap::new());
          add_import_to_graph(path, rule_index, &mut graph, &mut visited, &mut siblings);
        }
        Dependency::ICSSImportValue { prop, value } => {
          let (_, values) = imports
            .iter_mut()
            .last()
            .expect("test setup must produce the expected value");
          values.insert(value.to_string(), prop.to_string());
        }
        _ => {}
      }
    }
    let len = input.len() as u32;
    if index != len {
      result += Lexer::slice_range(input, &Range::new(index, len))
        .expect("test setup must produce the expected value");
    }
    let order = topological_sort(&graph, &mut warnings);
    for import in order {
      let values = &imports[import.as_str()];
      imported += ":import(\"";
      imported += &import;
      imported += "\") {\n";
      for (value, prop) in values {
        imported += "    ";
        imported += prop;
        imported += ": ";
        imported += &serialize_identifier(value);
        imported += ";\n";
      }
      imported += "}\n";
    }
    (imported + result.trim_start(), warnings)
  }
}

pub(crate) fn test(input: &str, expected: &str) {
  let (actual, warnings) = ExtractImports.transform(input);
  assert!(warnings.is_empty(), "{}", &warnings[0]);
  similar_asserts::assert_eq!(expected, actual);
}

pub(crate) fn test_with_warning(input: &str, expected: &str, warning: &str) {
  let (actual, warnings) = ExtractImports.transform(input);
  assert!(
    warnings[0].to_string().contains(warning),
    "{}",
    &warnings[0]
  );
  similar_asserts::assert_eq!(expected, actual);
}
#[test]
fn composing_globals() {
  test(
    ":local(.exportName) { composes: importName secondImport from global; other: rule; }",
    ":local(.exportName) { composes: global(importName) global(secondImport); other: rule; }",
  );
}

#[test]
fn existing_import() {
  test(
    indoc! {r#"
            :import("path/library.css") {
                something: else;
            }
            :local(.exportName) {
                composes: importName from 'path/library.css';
            }
        "#},
    indoc! {r#"
            :import("path/library.css") {
                something: else;
                i__imported_importName_0: importName;
            }
            :local(.exportName) {
                composes: i__imported_importName_0;
            }
        "#},
  );
}

#[test]
fn import_comment() {
  test(
    indoc! {r#"
            /*
            :local(.exportName) {
                composes: importName from "path/library.css";
                other: rule;
            }
            */
        "#},
    indoc! {r#"
            /*
            :local(.exportName) {
                composes: importName from "path/library.css";
                other: rule;
            }
            */
        "#},
  );
}

#[test]
fn import_consolidate() {
  test(
    indoc! {r#"
            :local(.exportName) {
                composes: importName secondImport from 'path/library.css';
                other: rule;
            }
            :local(.otherExport) {
                composes: thirdImport from 'path/library.css';
                composes: otherLibImport from 'path/other-lib.css';
            }
        "#},
    indoc! {r#"
            :import("path/library.css") {
                i__imported_importName_0: importName;
                i__imported_secondImport_1: secondImport;
                i__imported_thirdImport_2: thirdImport;
            }
            :import("path/other-lib.css") {
                i__imported_otherLibImport_3: otherLibImport;
            }
            :local(.exportName) {
                composes: i__imported_importName_0 i__imported_secondImport_1;
                other: rule;
            }
            :local(.otherExport) {
                composes: i__imported_thirdImport_2;
                composes: i__imported_otherLibImport_3;
            }
        "#},
  );
}

#[test]
fn import_local_extends() {
  test(
    indoc! {r#"
            :local(.exportName) {
                composes: localName;
                other: rule;
            }
        "#},
    indoc! {r#"
            :local(.exportName) {
                composes: localName;
                other: rule;
            }
        "#},
  );
}

#[test]
fn import_media() {
  test(
    indoc! {r#"
            @media screen {
                :local(.exportName) {
                    composes: importName from "path/library.css";
                    composes: importName2 from "path/library.css";
                    other: rule2;
                }
            }

            :local(.exportName) {
                composes: importName from "path/library.css";
                other: rule;
            }
        "#},
    indoc! {r#"
            :import("path/library.css") {
                i__imported_importName_0: importName;
                i__imported_importName2_1: importName2;
            }
            @media screen {
                :local(.exportName) {
                    composes: i__imported_importName_0;
                    composes: i__imported_importName2_1;
                    other: rule2;
                }
            }

            :local(.exportName) {
                composes: i__imported_importName_0;
                other: rule;
            }
        "#},
  );
}

#[test]
fn import_multiple_classes() {
  test(
    ":local(.exportName) { composes: importName secondImport from 'path/library.css'; other: rule; }\n",
    indoc! {r#"
            :import("path/library.css") {
                i__imported_importName_0: importName;
                i__imported_secondImport_1: secondImport;
            }
            :local(.exportName) { composes: i__imported_importName_0 i__imported_secondImport_1; other: rule; }
        "#},
  );
}

#[test]
fn import_multiple_references() {
  test(
    indoc! {r#"
            :local(.exportName) {
                composes: importName secondImport from 'path/library.css';
                composes: importName from 'path/library2.css';
                composes: importName2 from 'path/library.css';
            }
            :local(.exportName2) {
                composes: secondImport from 'path/library.css';
                composes: secondImport from 'path/library.css';
                composes: thirdDep from 'path/dep3.css';
            }
        "#},
    indoc! {r#"
            :import("path/library.css") {
                i__imported_importName_0: importName;
                i__imported_secondImport_1: secondImport;
                i__imported_importName2_3: importName2;
            }
            :import("path/library2.css") {
                i__imported_importName_2: importName;
            }
            :import("path/dep3.css") {
                i__imported_thirdDep_4: thirdDep;
            }
            :local(.exportName) {
                composes: i__imported_importName_0 i__imported_secondImport_1;
                composes: i__imported_importName_2;
                composes: i__imported_importName2_3;
            }
            :local(.exportName2) {
                composes: i__imported_secondImport_1;
                composes: i__imported_secondImport_1;
                composes: i__imported_thirdDep_4;
            }
        "#},
  );
}

#[test]
fn import_only_whitelist() {
  test(
    ":local(.exportName) { imports: importName from \"path/library.css\"; something-else: otherLibImport from \"path/other-lib.css\"; }",
    ":local(.exportName) { imports: importName from \"path/library.css\"; something-else: otherLibImport from \"path/other-lib.css\"; }",
  );
}

#[test]
fn import_preserving_order() {
  test(
    indoc! {r#"
            .a {
                composes: b from "./b.css";
                composes: c from "./c.css";
                color: #aaa;
            }
        "#},
    indoc! {r#"
            :import("./b.css") {
                i__imported_b_0: b;
            }
            :import("./c.css") {
                i__imported_c_1: c;
            }
            .a {
                composes: i__imported_b_0;
                composes: i__imported_c_1;
                color: #aaa;
            }
        "#},
  );
}

#[test]
fn import_single_quotes() {
  test(
    indoc! {r#"
            :local(.exportName) {
                composes: importName from 'path/library.css';
                other: rule;
            }
        "#},
    indoc! {r#"
            :import("path/library.css") {
                i__imported_importName_0: importName;
            }
            :local(.exportName) {
                composes: i__imported_importName_0;
                other: rule;
            }
        "#},
  );
}

#[test]
fn import_spacing() {
  test(
    indoc! {r#"
            :local(.exportName) {
                composes: importName  from   	"path/library.css";
                composes: importName2 from   	"path/library.css";
                composes: importName   importName2   from   "path/library.css";
                other: rule;
            }
        "#},
    indoc! {r#"
            :import("path/library.css") {
                i__imported_importName_0: importName;
                i__imported_importName2_1: importName2;
            }
            :local(.exportName) {
                composes: i__imported_importName_0;
                composes: i__imported_importName2_1;
                composes: i__imported_importName_0 i__imported_importName2_1;
                other: rule;
            }
        "#},
  );
}

#[test]
fn import_within() {
  test(
    indoc! {r#"
            :local(.exportName) {
                composes: importName from "path/library.css";
                other: rule;
            }
        "#},
    indoc! {r#"
            :import("path/library.css") {
                i__imported_importName_0: importName;
            }
            :local(.exportName) {
                composes: i__imported_importName_0;
                other: rule;
            }
        "#},
  );
}

#[test]
fn multiple_composes() {
  test(
    indoc! {r#"
            :local(.exportName) {
                composes: importName from "path/library.css", beforeName from global, importName secondImport from global, firstImport secondImport from "path/library.css";
                other: rule;
            }

            :local(.duplicate) {
                composes: a from "./aa.css", b from "./bb.css", c from "./cc.css", a from "./aa.css", c from "./cc.css";
            }

            :local(.spaces) {
                composes: importName importName2 from "path/library.css", importName3 importName4 from "path/library.css";
            }

            :local(.unknown) {
                composes: foo bar, baz;
            }
        "#},
    indoc! {r#"
            :import("path/library.css") {
                i__imported_importName_0: importName;
                i__imported_firstImport_1: firstImport;
                i__imported_secondImport_2: secondImport;
                i__imported_importName2_6: importName2;
                i__imported_importName3_7: importName3;
                i__imported_importName4_8: importName4;
            }
            :import("./aa.css") {
                i__imported_a_3: a;
            }
            :import("./bb.css") {
                i__imported_b_4: b;
            }
            :import("./cc.css") {
                i__imported_c_5: c;
            }
            :local(.exportName) {
                composes: i__imported_importName_0, global(beforeName), global(importName) global(secondImport), i__imported_firstImport_1 i__imported_secondImport_2;
                other: rule;
            }

            :local(.duplicate) {
                composes: i__imported_a_3, i__imported_b_4, i__imported_c_5, i__imported_a_3, i__imported_c_5;
            }

            :local(.spaces) {
                composes: i__imported_importName_0 i__imported_importName2_6, i__imported_importName3_7 i__imported_importName4_8;
            }

            :local(.unknown) {
                composes: foo bar, baz;
            }
        "#},
  );
}
#[test]
fn nesting() {
  test_with_warning(
    indoc! {r#"
            :local(.foo) {
                display: grid;

                @media (orientation: landscape) {
                    &:local(.bar) {
                        grid-auto-flow: column;

                        @media (min-width: 1024px) {
                            &:local(.baz) {
                                composes: importName from "path/library.css";
                            }
                        }
                    }
                }
            }
        "#},
    indoc! {r#"
            :local(.foo) {
                display: grid;

                @media (orientation: landscape) {
                    &:local(.bar) {
                        grid-auto-flow: column;

                        @media (min-width: 1024px) {
                            &:local(.baz) {
                                composes: importName from "path/library.css";
                            }
                        }
                    }
                }
            }
        "#},
    "Composition is not allowed in nested rule",
  );
}

#[test]
fn resolve_composes_order() {
  test(
    indoc! {r#"
            .a {
                composes: c from "./c.css";
                color: #bebebe;
            }

            .b {
                /* `b` should be after `c` */
                composes: b from "./b.css";
                composes: c from "./c.css";
                color: #aaa;
            }
        "#},
    indoc! {r#"
            :import("./b.css") {
                i__imported_b_1: b;
            }
            :import("./c.css") {
                i__imported_c_0: c;
            }
            .a {
                composes: i__imported_c_0;
                color: #bebebe;
            }

            .b {
                /* `b` should be after `c` */
                composes: i__imported_b_1;
                composes: i__imported_c_0;
                color: #aaa;
            }
        "#},
  );
}

#[test]
fn resolve_duplicates() {
  test(
    indoc! {r#"
            :import("./cc.css") {
                smthing: somevalue;
            }

            .a {
                composes: a from './aa.css';
                composes: b from './bb.css';
                composes: c from './cc.css';
                composes: a from './aa.css';
                composes: c from './cc.css';
            }
        "#},
    indoc! {r#"
            :import("./aa.css") {
                i__imported_a_0: a;
            }
            :import("./bb.css") {
                i__imported_b_1: b;
            }
            :import("./cc.css") {
                smthing: somevalue;
                i__imported_c_2: c;
            }
            .a {
                composes: i__imported_a_0;
                composes: i__imported_b_1;
                composes: i__imported_c_2;
                composes: i__imported_a_0;
                composes: i__imported_c_2;
            }
        "#},
  );
}

#[test]
fn resolve_imports_order() {
  test(
    indoc! {r#"
            :import("custom-path.css") {
                /* empty to check the order */
            }

            :import("./bb.css") {
                somevalue: localvalue;
            }

            .a {
                composes: aa from './aa.css';
            }

            .b {
                composes: bb from './bb.css';
                composes: bb from './aa.css';
            }

            .c {
                composes: cc from './cc.css';
                composes: cc from './aa.css';
            }

            .d {
                composes: dd from './cc.css';
                composes: dd from './bb.css';
                composes: dd from './dd.css';
            }
        "#},
    indoc! {r#"
            :import("custom-path.css") {
            }
            :import("./cc.css") {
                i__imported_cc_3: cc;
                i__imported_dd_5: dd;
            }
            :import("./bb.css") {
                somevalue: localvalue;
                i__imported_bb_1: bb;
                i__imported_dd_6: dd;
            }
            :import("./aa.css") {
                i__imported_aa_0: aa;
                i__imported_bb_2: bb;
                i__imported_cc_4: cc;
            }
            :import("./dd.css") {
                i__imported_dd_7: dd;
            }
            .a {
                composes: i__imported_aa_0;
            }

            .b {
                composes: i__imported_bb_1;
                composes: i__imported_bb_2;
            }

            .c {
                composes: i__imported_cc_3;
                composes: i__imported_cc_4;
            }

            .d {
                composes: i__imported_dd_5;
                composes: i__imported_dd_6;
                composes: i__imported_dd_7;
            }
        "#},
  );
}

#[test]
fn valid_characters() {
  test(
    indoc! {r#"
            :local(.exportName) {
                composes: a -b --c _d from "path/library.css";
                composes: a_ b- c-- d\% from "path/library2.css";
            }
        "#},
    indoc! {r#"
            :import("path/library.css") {
                i__imported_a_0: a;
                i__imported__b_1: -b;
                i__imported___c_2: --c;
                i__imported__d_3: _d;
            }
            :import("path/library2.css") {
                i__imported_a__4: a_;
                i__imported_b__5: b-;
                i__imported_c___6: c--;
                i__imported_d___7: d\\\%;
            }
            :local(.exportName) {
                composes: i__imported_a_0 i__imported__b_1 i__imported___c_2 i__imported__d_3;
                composes: i__imported_a__4 i__imported_b__5 i__imported_c___6 i__imported_d___7;
            }
        "#},
  );
}

#[test]
fn check_import_order() {
  test_with_warning(
    indoc! {r#"
            .aa {
                composes: b from './b.css';
                composes: c from './c.css';
            }

            .bb {
                composes: c from './c.css';
                composes: b from './b.css';
            }
        "#},
    indoc! {r#"
            :import("./c.css") {
                i__imported_c_1: c;
            }
            :import("./b.css") {
                i__imported_b_0: b;
            }
            .aa {
                composes: i__imported_b_0;
                composes: i__imported_c_1;
            }

            .bb {
                composes: i__imported_c_1;
                composes: i__imported_b_0;
            }
        "#},
    "Failed to resolve order of composed modules",
  );
}

mod topological_sort {
  use super::*;

  #[test]
  fn should_resolve_graphs() {
    let mut warnings = Vec::new();
    let graph: LinkedHashMap<String, Vec<String>> = LinkedHashMap::from_iter([
      ("v1".into(), vec!["v2".into(), "v5".into()]),
      ("v2".into(), vec![]),
      ("v3".into(), vec!["v2".into(), "v4".into(), "v5".into()]),
      ("v4".into(), vec![]),
      ("v5".into(), vec![]),
    ]);
    let order = topological_sort(&graph, &mut warnings);
    assert_eq!(order, vec!["v2", "v5", "v1", "v4", "v3"]);
    assert!(warnings.is_empty());
    let graph: LinkedHashMap<String, Vec<String>> = LinkedHashMap::from_iter([
      ("v1".into(), vec!["v2".into(), "v5".into()]),
      ("v2".into(), vec!["v4".into()]),
      ("v3".into(), vec!["v2".into(), "v4".into(), "v5".into()]),
      ("v4".into(), vec![]),
      ("v5".into(), vec![]),
    ]);
    let order = topological_sort(&graph, &mut warnings);
    assert_eq!(order, vec!["v4", "v2", "v5", "v1", "v3"]);
    assert!(warnings.is_empty());
  }

  #[test]
  fn cycle_in_the_graph() {
    let mut warnings = Vec::new();
    let graph: LinkedHashMap<String, Vec<String>> = LinkedHashMap::from_iter([
      ("v1".into(), vec!["v3".into()]),
      ("v2".into(), vec![]),
      ("v3".into(), vec!["v1".into()]),
    ]);
    let order = topological_sort(&graph, &mut warnings);
    assert_eq!(order, vec!["v3", "v1", "v2"]);
    assert!(!warnings.is_empty());
  }
}
