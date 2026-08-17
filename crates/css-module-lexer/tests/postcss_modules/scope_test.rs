use css_module_lexer::{Dependency, Lexer, Mode, Range, Warning, collect_dependencies};
use indoc::indoc;
use linked_hash_map::LinkedHashMap;

#[derive(Debug, Default, Clone, Hash, PartialEq, Eq)]
struct Scope;

const WHITESPACE_MARKER: &str = "__CSS_MODULE_LEXER_WHITESPACE__";

fn generate_local_name(name: &str) -> String {
  format!("_input__{name}")
}

fn strip_whitespace_markers(value: &str) -> String {
  value.split(WHITESPACE_MARKER).collect()
}

impl Scope {
  pub fn transform<'s>(&self, input: &'s str) -> (String, Vec<Warning<'s>>) {
    let mut result = String::new();
    let mut index = 0;
    let mut exports = LinkedHashMap::new();
    let (dependencies, warnings) = collect_dependencies(input, Mode::Global);
    for dependency in &dependencies {
      match dependency {
        Dependency::LocalClass { name, range, .. } => {
          result += Lexer::slice_range(input, &Range::new(index, range.start))
            .expect("test setup must produce the expected value");
          let name = &name[1..];
          result += ".";
          let new_name = generate_local_name(name);
          result += &new_name;
          exports.insert(name.to_string(), vec![new_name]);
          index = range.end;
        }
        Dependency::LocalId { name, range, .. } => {
          result += Lexer::slice_range(input, &Range::new(index, range.start))
            .expect("test setup must produce the expected value");
          let name = &name[1..];
          result += "#";
          let new_name = generate_local_name(name);
          result += &new_name;
          exports.insert(name.to_string(), vec![new_name]);
          index = range.end;
        }
        Dependency::LocalKeyframes { name, range } => {
          result += Lexer::slice_range(input, &Range::new(index, range.start))
            .expect("test setup must produce the expected value");
          let new_name = generate_local_name(name);
          result += &new_name;
          exports.insert(name.to_string(), vec![new_name]);
          index = range.end;
        }
        Dependency::LocalKeyframesDecl { name, range } => {
          result += Lexer::slice_range(input, &Range::new(index, range.start))
            .expect("test setup must produce the expected value");
          let new_name = generate_local_name(name);
          result += &new_name;
          exports.insert(name.to_string(), vec![new_name]);
          index = range.end;
        }
        Dependency::Composes {
          local_classes,
          names,
          from,
          ..
        } => {
          let is_global = *from == Some("global");
          let names = dependencies.composes_names(*names);
          let local_classes = dependencies.composes_local_classes(*local_classes);
          for name in names {
            let name = *name;
            let new_name = if is_global {
              name.to_string()
            } else {
              generate_local_name(name)
            };
            for local_class in local_classes {
              let local_class = *local_class;
              if let Some(existing) = exports.get(name) {
                let existing = existing.clone();
                exports
                  .get_mut(local_class)
                  .expect("test setup must produce the expected value")
                  .extend(existing);
              } else {
                exports
                  .get_mut(local_class)
                  .expect("test setup must produce the expected value")
                  .push(new_name.clone());
              }
            }
          }
        }
        Dependency::Replace { content, range } => {
          let original =
            Lexer::slice_range(input, range).expect("test setup must produce the expected value");
          if original.starts_with(":export") || original.starts_with(":import(") {
            continue;
          }
          result += Lexer::slice_range(input, &Range::new(index, range.start))
            .expect("test setup must produce the expected value");
          result += content;
          index = range.end;
        }
        _ => {}
      }
    }
    let len = input.len() as u32;
    if index != len {
      result += Lexer::slice_range(input, &Range::new(index, len))
        .expect("test setup must produce the expected value");
    }
    if !exports.is_empty() {
      result += "\n:export {\n";
      for (key, value) in exports {
        result += "    ";
        result += &key;
        result += ": ";
        result += &value.join(" ");
        result += ";\n";
      }
      result += "}\n";
    }
    (result, warnings)
  }
}

fn test(input: &str, expected: &str) {
  let input = strip_whitespace_markers(input);
  let expected = strip_whitespace_markers(expected);
  let (actual, warnings) = Scope.transform(&input);
  assert!(warnings.is_empty(), "{}", &warnings[0]);
  similar_asserts::assert_eq!(expected, actual);
}

fn test_with_warning(input: &str, expected: &str, warning: &str) {
  let input = strip_whitespace_markers(input);
  let expected = strip_whitespace_markers(expected);
  let (actual, warnings) = Scope.transform(&input);
  assert!(
    warnings[0].to_string().contains(warning),
    "{}",
    &warnings[0]
  );
  similar_asserts::assert_eq!(expected, actual);
}

#[test]
fn at_rule() {
  test(
    indoc! {r#"
            :local(.otherClass) {
                background: red;
            }

            @media screen {
                :local(.foo) {
                    color: green;
                    :local(.baz) {
                        color: blue;
                    }
                }
            }
        "#},
    indoc! {r#"
            ._input__otherClass {
                background: red;
            }

            @media screen {
                ._input__foo {
                    color: green;
                    ._input__baz {
                        color: blue;
                    }
                }
            }

            :export {
                otherClass: _input__otherClass;
                foo: _input__foo;
                baz: _input__baz;
            }
        "#},
  );
}

#[test]
fn at_rule_scope() {
  test(
    indoc! {r#"
            :local(.d) {
                color: red;
            }
            __CSS_MODULE_LEXER_WHITESPACE__
            @scope (:local(.a)) to (:local(.b)) {
                :local(.c) {
                    border: 5px solid black;
                    background-color: goldenrod;
                }
            }
            __CSS_MODULE_LEXER_WHITESPACE__
            @scope (:local(.a)) {
                :local(.e) {
                    border: 5px solid black;
                }
            }
            __CSS_MODULE_LEXER_WHITESPACE__
            @scope (:local(.a)) to (img) {
                :local(.f) {
                    background-color: goldenrod;
                }
            }
            __CSS_MODULE_LEXER_WHITESPACE__
            @scope (:local(.g)) {
                img {
                    backdrop-filter: blur(2px);
                }
            }
            __CSS_MODULE_LEXER_WHITESPACE__
            @scope {
                :scope {
                    color: red;
                }
            }
        "#},
    indoc! {r#"
            ._input__d {
                color: red;
            }
            __CSS_MODULE_LEXER_WHITESPACE__
            @scope (._input__a) to (._input__b) {
                ._input__c {
                    border: 5px solid black;
                    background-color: goldenrod;
                }
            }
            __CSS_MODULE_LEXER_WHITESPACE__
            @scope (._input__a) {
                ._input__e {
                    border: 5px solid black;
                }
            }
            __CSS_MODULE_LEXER_WHITESPACE__
            @scope (._input__a) to (img) {
                ._input__f {
                    background-color: goldenrod;
                }
            }
            __CSS_MODULE_LEXER_WHITESPACE__
            @scope (._input__g) {
                img {
                    backdrop-filter: blur(2px);
                }
            }
            __CSS_MODULE_LEXER_WHITESPACE__
            @scope {
                :scope {
                    color: red;
                }
            }
            __CSS_MODULE_LEXER_WHITESPACE__
            :export {
                d: _input__d;
                b: _input__b;
                c: _input__c;
                e: _input__e;
                a: _input__a;
                f: _input__f;
                g: _input__g;
            }
        "#},
  );
}

#[test]
fn composes_only_allowed() {
  test(
    indoc! {r#"
            :local(.class) {
                composes: global(a);
                compose-with: global(b);
                a-composes: global(c);
                composes-b: global(d);
                a-composes-b: global(e);
                a-compose-with-b: global(b);
            }
        "#},
    indoc! {r#"
            ._input__class {
                __CSS_MODULE_LEXER_WHITESPACE__
                __CSS_MODULE_LEXER_WHITESPACE__
                a-composes: global(c);
                composes-b: global(d);
                a-composes-b: global(e);
                a-compose-with-b: global(b);
            }

            :export {
                class: _input__class a b;
            }
        "#},
  );
}

#[test]
fn css_nesting() {
  test(
    indoc! {r#"
            :local(.otherClass) {
                background: red;
            }

            :local(.foo) {
                color: green;

                @media (max-width: 520px) {
                    :local(.bar) {
                        color: darkgreen;
                    }

                    &:local(.baz) {
                        color: blue;
                    }
                }
            }

            :local(.a) {
                color: red;

                &:local(.b) {
                    color: green;
                }

                :local(.c) {
                    color: blue;
                }
            }
        "#},
    indoc! {r#"
            ._input__otherClass {
                background: red;
            }

            ._input__foo {
                color: green;

                @media (max-width: 520px) {
                    ._input__bar {
                        color: darkgreen;
                    }

                    &._input__baz {
                        color: blue;
                    }
                }
            }

            ._input__a {
                color: red;

                &._input__b {
                    color: green;
                }

                ._input__c {
                    color: blue;
                }
            }

            :export {
                otherClass: _input__otherClass;
                foo: _input__foo;
                bar: _input__bar;
                baz: _input__baz;
                a: _input__a;
                b: _input__b;
                c: _input__c;
            }
        "#},
  );
}

#[test]
fn css_nesting_composes() {
  test(
    indoc! {r#"
            :local(.bar) {
                color: red;
            }

            :local(.foo) {
                display: grid;
                composes: bar;

                @media (orientation: landscape) {
                    grid-auto-flow: column;
                }
            }
        "#},
    indoc! {r#"
            ._input__bar {
                color: red;
            }

            ._input__foo {
                display: grid;
                __CSS_MODULE_LEXER_WHITESPACE__

                @media (orientation: landscape) {
                    grid-auto-flow: column;
                }
            }

            :export {
                bar: _input__bar;
                foo: _input__foo _input__bar;
            }
        "#},
  );
}

#[test]
fn css_nesting_composes_with_nested_media() {
  test(
    indoc! {r#"
            :local(.a) {
            }

            @media (orientation: landscape) {
                @media (orientation: landscape) {
                    :local(.b) {
                        color: red;
                        composes: a;
                    }
                }
            }
        "#},
    indoc! {r#"
            ._input__a {
            }

            @media (orientation: landscape) {
                @media (orientation: landscape) {
                    ._input__b {
                        color: red;
                        __CSS_MODULE_LEXER_WHITESPACE__
                    }
                }
            }

            :export {
                a: _input__a;
                b: _input__b _input__a;
            }
        "#},
  );
}

#[test]
fn error_comma_in_local() {
  test(
    indoc! {r#"
            :local(.a, .b) {
                composes: className;
            }
        "#},
    indoc! {r#"
            ._input__a, ._input__b {
                __CSS_MODULE_LEXER_WHITESPACE__
            }

            :export {
                a: _input__a _input__className;
                b: _input__b _input__className;
            }
        "#},
  );
}

#[test]
fn error_composes_css_nesting() {
  test_with_warning(
    indoc! {r#"
            :local(.otherClassName) {
            }

            :local(.a) {
                :local(.b) {
                    compose-with: otherClassName;
                }
            }
        "#},
    indoc! {r#"
            ._input__otherClassName {
            }

            ._input__a {
                ._input__b {
                    compose-with: otherClassName;
                }
            }

            :export {
                otherClassName: _input__otherClassName;
                a: _input__a;
                b: _input__b;
            }
        "#},
    "Composition is not allowed in nested rule",
  );
}

#[test]
fn error_composes_css_nesting_at_rule() {
  test_with_warning(
    indoc! {r#"
            :local(.otherClassName) {
            }

            @media (min-width: 1024px) {
                :local(.a) {
                    :local(.b) {
                        compose-with: otherClassName;
                    }
                }
            }
        "#},
    indoc! {r#"
            ._input__otherClassName {
            }

            @media (min-width: 1024px) {
                ._input__a {
                    ._input__b {
                        compose-with: otherClassName;
                    }
                }
            }

            :export {
                otherClassName: _input__otherClassName;
                a: _input__a;
                b: _input__b;
            }
        "#},
    "Composition is not allowed in nested rule",
  );
}

#[test]
fn error_composes_css_nesting_with_media() {
  test_with_warning(
    indoc! {r#"
            :local(.otherClassName) {
            }

            :local(.a) {
                @media (min-width: 1024px) {
                    :local(.b) {
                        compose-with: otherClassName;
                    }
                }
            }
        "#},
    indoc! {r#"
            ._input__otherClassName {
            }

            ._input__a {
                @media (min-width: 1024px) {
                    ._input__b {
                        compose-with: otherClassName;
                    }
                }
            }

            :export {
                otherClassName: _input__otherClassName;
                a: _input__a;
                b: _input__b;
            }
        "#},
    "Composition is not allowed in nested rule",
  );
}

#[test]
fn error_composes_keyframes() {
  test_with_warning(
    indoc! {r#"
            :local(.bar) {
            }

            @keyframes slidein {
                from {
                    transform: translateX(0%);
                }
                __CSS_MODULE_LEXER_WHITESPACE__
                to {
                    composes: bar;
                }
            }
        "#},
    indoc! {r#"
            ._input__bar {
            }

            @keyframes slidein {
                from {
                    transform: translateX(0%);
                }
                __CSS_MODULE_LEXER_WHITESPACE__
                to {
                    composes: bar;
                }
            }
            __CSS_MODULE_LEXER_WHITESPACE__
            :export {
                bar: _input__bar;
            }
        "#},
    "Composition is only allowed when selector is single :local class",
  );
}

#[test]
fn error_composes_not_allowed_in_local_id() {
  test_with_warning(
    indoc! {r#"
            :local(#idName) {
                composes: className;
            }
        "#},
    indoc! {r#"
            #_input__idName {
                composes: className;
            }

            :export {
                idName: _input__idName;
            }
        "#},
    "Composition is only allowed when selector is single :local class",
  );
}

#[test]
fn error_composes_not_allowed_in_multiple() {
  test_with_warning(
    indoc! {r#"
            :local(.a) :local(.b) {
                composes: className;
            }
        "#},
    indoc! {r#"
            ._input__a ._input__b {
                composes: className;
            }

            :export {
                a: _input__a;
                b: _input__b;
            }
        "#},
    "Composition is only allowed when selector is single :local class",
  );
}

#[test]
fn error_composes_not_allowed_in_simple() {
  test_with_warning(
    indoc! {r#"
            body {
                composes: className;
            }
        "#},
    indoc! {r#"
            body {
                composes: className;
            }
        "#},
    "Composition is only allowed when selector is single :local class",
  );
}

#[test]
fn error_composes_not_allowed_in_wrong_local() {
  test_with_warning(
    indoc! {r#"
            :local(.a.b) {
                composes: className;
            }
        "#},
    indoc! {r#"
            ._input__a._input__b {
                composes: className;
            }

            :export {
                a: _input__a;
                b: _input__b;
            }
        "#},
    "Composition is only allowed when selector is single :local class",
  );
}

#[test]
fn error_composes_not_defined_class() {
  // TODO: should warning for otherClassName not found
  test(
    indoc! {r#"
            :local(.className) {
                compose-with: otherClassName;
            }
        "#},
    indoc! {r#"
            ._input__className {
                __CSS_MODULE_LEXER_WHITESPACE__
            }

            :export {
                className: _input__className _input__otherClassName;
            }
        "#},
  );
}

#[test]
fn error_multiple_nested_media() {
  test_with_warning(
    indoc! {r#"
            :local(.bar) {
                color: blue;
            }

            :local(.foo) {
                display: grid;

                @media (orientation: landscape) {
                    grid-auto-flow: column;

                    @media (min-width: 1024px) {
                        composes: bar;
                    }
                }
            }
        "#},
    indoc! {r#"
            ._input__bar {
                color: blue;
            }

            ._input__foo {
                display: grid;
            __CSS_MODULE_LEXER_WHITESPACE__
                @media (orientation: landscape) {
                    grid-auto-flow: column;
            __CSS_MODULE_LEXER_WHITESPACE__
                    @media (min-width: 1024px) {
                        composes: bar;
                    }
                }
            }

            :export {
                bar: _input__bar;
                foo: _input__foo;
            }
        "#},
    "Composition is not allowed in nested rule",
  );
}

#[test]
fn error_not_allowed_in_local() {
  // TODO: validate selector, should warning for :local(body)
  test(
    indoc! {r#"
            :local(body) {
                color: red;
            }
        "#},
    indoc! {r#"
            body {
                color: red;
            }
        "#},
  );
}

#[test]
fn error_when_attribute_is_href() {
  // TODO: validate selector, should warning for :local(.exportName1[href^="https"])
  test(
    indoc! {r#"
            :local(.exportName1[href^="https"]) {
                color: blue;
            }
        "#},
    indoc! {r#"
            ._input__exportName1[href^="https"] {
                color: blue;
            }

            :export {
                exportName1: _input__exportName1;
            }
        "#},
  );
}

#[test]
fn error_when_attribute_is_target() {
  // TODO: validate selector, should warning for :local(.exportName1[target="_blank"])
  test(
    indoc! {r#"
            :local(.exportName1[target="_blank"]) {
                color: blue;
            }
        "#},
    indoc! {r#"
            ._input__exportName1[target="_blank"] {
                color: blue;
            }

            :export {
                exportName1: _input__exportName1;
            }
        "#},
  );
}

#[test]
fn error_when_attribute_is_title() {
  // TODO: validate selector, should warning for :local(.exportName1[title="flower"])
  test(
    indoc! {r#"
            :local(.exportName1[title="flower"]) {
                color: blue;
            }
        "#},
    indoc! {r#"
            ._input__exportName1[title="flower"] {
                color: blue;
            }

            :export {
                exportName1: _input__exportName1;
            }
        "#},
  );
}

#[test]
fn error_when_attribute_is_type() {
  // TODO: validate selector, should warning for :local(.exportName1[type="text"])
  test(
    indoc! {r#"
            :local(.exportName1[type="text"]) {
                color: blue;
            }
        "#},
    indoc! {r#"
            ._input__exportName1[type="text"] {
                color: blue;
            }

            :export {
                exportName1: _input__exportName1;
            }
        "#},
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
