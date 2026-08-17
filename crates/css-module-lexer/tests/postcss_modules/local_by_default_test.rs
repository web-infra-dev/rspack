use std::collections::HashSet;

use css_module_lexer::{Dependency, Lexer, Mode, Range, Warning, collect_dependencies};
use indoc::indoc;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct LocalByDefault {
  pub mode: Mode,
}

impl LocalByDefault {
  pub fn transform<'s>(&self, input: &'s str) -> (String, Vec<Warning<'s>>) {
    let mut result = String::new();
    let mut index = 0;
    let mut local_alias: HashSet<String> = HashSet::new();
    let add_local = |result: &mut String, name: &str| {
      *result += ":local(";
      *result += name;
      *result += ")";
    };
    let (dependencies, warnings) = collect_dependencies(input, self.mode);
    for dependency in &dependencies {
      match dependency {
        Dependency::LocalClass {
          name,
          range,
          explicit,
        }
        | Dependency::LocalId {
          name,
          range,
          explicit,
        } => {
          if !explicit && local_alias.contains(&name[1..]) {
            continue;
          }
          result += Lexer::slice_range(input, &Range::new(index, range.start))
            .expect("test setup must produce the expected value");
          add_local(
            &mut result,
            Lexer::slice_range(input, range).expect("test setup must produce the expected value"),
          );
          index = range.end;
        }
        Dependency::LocalKeyframes { name, range } => {
          if local_alias.contains(*name) {
            continue;
          }
          result += Lexer::slice_range(input, &Range::new(index, range.start))
            .expect("test setup must produce the expected value");
          add_local(
            &mut result,
            Lexer::slice_range(input, range).expect("test setup must produce the expected value"),
          );
          index = range.end;
        }
        Dependency::LocalKeyframesDecl { range, .. } => {
          result += Lexer::slice_range(input, &Range::new(index, range.start))
            .expect("test setup must produce the expected value");
          add_local(
            &mut result,
            Lexer::slice_range(input, range).expect("test setup must produce the expected value"),
          );
          index = range.end;
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
        Dependency::ICSSImportValue { prop, .. } => {
          local_alias.insert(prop.to_string());
        }
        _ => {}
      }
    }
    let len = input.len() as u32;
    if index != len {
      result += Lexer::slice_range(input, &Range::new(index, len))
        .expect("test setup must produce the expected value");
    }
    (result, warnings)
  }
}

fn test(input: &str, expected: &str) {
  let (actual, warnings) = LocalByDefault { mode: Mode::Local }.transform(input);
  assert!(warnings.is_empty(), "{}", &warnings[0]);
  similar_asserts::assert_eq!(expected, actual);
}

fn test_with_options(input: &str, expected: &str, options: LocalByDefault) {
  let (actual, warnings) = options.transform(input);
  assert!(warnings.is_empty(), "{}", &warnings[0]);
  similar_asserts::assert_eq!(expected, actual);
}

fn test_with_warning(input: &str, expected: &str, warning: &str) {
  let (actual, warnings) = LocalByDefault { mode: Mode::Local }.transform(input);
  assert!(!warnings.is_empty(), "missing warning for {input:?}");
  assert!(
    warnings[0].to_string().contains(warning),
    "{}",
    &warnings[0]
  );
  similar_asserts::assert_eq!(expected, actual);
}

fn test_with_options_warning(input: &str, expected: &str, options: LocalByDefault, warning: &str) {
  let (actual, warnings) = options.transform(input);
  assert!(
    warnings[0].to_string().contains(warning),
    "{}",
    &warnings[0]
  );
  similar_asserts::assert_eq!(expected, actual);
}

#[test]
fn scope_selectors() {
  test(".foobar {}", ":local(.foobar) {}");
}

#[test]
fn scope_escaped_selectors() {
  test(".\\3A \\) {}", ":local(.\\3A \\)) {}");
}

#[test]
fn scope_ids() {
  test("#foobar {}", ":local(#foobar) {}");
}

#[test]
fn scope_escaped_ids() {
  test("#\\#test {}", ":local(#\\#test) {}");
  test("#u-m\\00002b {}", ":local(#u-m\\00002b ){}");
}

#[test]
fn scope_multiple_selectors() {
  test(".foo, .baz {}", ":local(.foo), :local(.baz) {}");
}

#[test]
fn scope_sibling_selectors() {
  test(".foo ~ .baz {}", ":local(.foo) ~ :local(.baz) {}");
}

#[test]
fn scope_psuedo_elements() {
  test(".foo:after {}", ":local(.foo):after {}");
}

#[test]
fn scope_media_queries() {
  test(
    "@media only screen { .foo {} }",
    "@media only screen { :local(.foo) {} }",
  );
}

#[test]
fn allow_narrow_global_selectors() {
  test(":global(.foo .bar) {}", ".foo .bar {}");
}

#[test]
fn allow_narrow_local_selectors() {
  test(":local(.foo .bar) {}", ":local(.foo) :local(.bar) {}");
}

#[test]
fn allow_broad_global_selectors() {
  test(":global .foo .bar {}", ".foo .bar {}");
}

#[test]
fn allow_broad_local_selectors() {
  test(":local .foo .bar {}", ":local(.foo) :local(.bar) {}");
}

#[test]
fn allow_multiple_narrow_global_selectors() {
  test(":global(.foo), :global(.bar) {}", ".foo, .bar {}");
}

#[test]
fn allow_multiple_broad_global_selectors() {
  test(":global .foo, :global .bar {}", ".foo, .bar {}");
}

#[test]
fn allow_multiple_broad_local_selectors() {
  test(
    ":local .foo, :local .bar {}",
    ":local(.foo), :local(.bar) {}",
  );
}

#[test]
fn allow_narrow_global_selectors_nested_inside_local_styles() {
  test(".foo :global(.foo .bar) {}", ":local(.foo) .foo .bar {}");
}

#[test]
fn allow_broad_global_selectors_nested_inside_local_styles() {
  test(".foo :global .foo .bar {}", ":local(.foo) .foo .bar {}");
}

#[test]
fn allow_parentheses_inside_narrow_global_selectors() {
  test(
    ".foo :global(.foo:not(.bar)) {}",
    ":local(.foo) .foo:not(.bar) {}",
  );
}

#[test]
fn allow_parentheses_inside_narrow_local_selectors() {
  test(
    ".foo :local(.foo:not(.bar)) {}",
    ":local(.foo) :local(.foo):not(:local(.bar)) {}",
  );
}

#[test]
fn allow_narrow_global_selectors_appended_to_local_styles() {
  test(".foo:global(.foo.bar) {}", ":local(.foo).foo.bar {}");
}

#[test]
fn ignore_selectors_that_are_already_local() {
  test(":local(.foobar) {}", ":local(.foobar) {}");
}

#[test]
fn ignore_nested_selectors_that_are_already_local() {
  test(
    ":local(.foo) :local(.bar) {}",
    ":local(.foo) :local(.bar) {}",
  );
}

#[test]
fn ignore_multiple_selectors_that_are_already_local() {
  test(
    ":local(.foo), :local(.bar) {}",
    ":local(.foo), :local(.bar) {}",
  );
}

#[test]
fn ignore_sibling_selectors_that_are_already_local() {
  test(
    ":local(.foo) ~ :local(.bar) {}",
    ":local(.foo) ~ :local(.bar) {}",
  );
}

#[test]
fn ignore_psuedo_elements_that_are_already_local() {
  test(":local(.foo):after {}", ":local(.foo):after {}");
}

#[test]
fn trim_whitespace_after_empty_broad_selector() {
  test(".bar :global :global {}", ":local(.bar) {}");
}

#[test]
fn broad_global_should_be_limited_to_selector() {
  test(
    ":global .foo, .bar :global, .foobar :global {}",
    // should be ".foo, :local(.bar), :local(.foobar) {}", but the whitespace is fine
    ".foo, :local(.bar) , :local(.foobar) {}",
  );
}

#[test]
fn broad_global_should_be_limited_to_nested_selector() {
  test(
    ".foo:not(:global .bar).foobar {}",
    ":local(.foo):not(.bar):local(.foobar) {}",
  );
}

#[test]
fn broad_global_and_local_should_allow_switching() {
  test(
    ".foo :global .bar :local .foobar :local .barfoo {}",
    ":local(.foo) .bar :local(.foobar) :local(.barfoo) {}",
  );
}

#[test]
fn localize_a_single_animation_name() {
  test(
    ".foo { animation-name: bar; }",
    ":local(.foo) { animation-name: :local(bar); }",
  );
}

#[test]
fn not_localize_animation_name_in_a_var_function() {
  test(
    ".foo { animation-name: var(--bar); }",
    ":local(.foo) { animation-name: var(--bar); }",
  );
  test(
    ".foo { animation-name: vAr(--bar); }",
    ":local(.foo) { animation-name: vAr(--bar); }",
  );
}

#[test]
fn not_localize_animation_name_in_an_env_function() {
  test(
    ".foo { animation-name: env(bar); }",
    ":local(.foo) { animation-name: env(bar); }",
  );
  test(
    ".foo { animation-name: eNv(bar); }",
    ":local(.foo) { animation-name: eNv(bar); }",
  );
}

#[test]
fn not_localize_a_single_animation_delay() {
  test(
    ".foo { animation-delay: 1s; }",
    ":local(.foo) { animation-delay: 1s; }",
  );
}

#[test]
fn localize_multiple_animation_names() {
  test(
    ".foo { animation-name: bar, foobar; }",
    ":local(.foo) { animation-name: :local(bar), :local(foobar); }",
  );
}

#[test]
fn not_localize_revert() {
  test(
    ".foo { animation: revert; }",
    ":local(.foo) { animation: revert; }",
  );
  test(
    ".foo { animation-name: revert; }",
    ":local(.foo) { animation-name: revert; }",
  );
  test(
    ".foo { animation-name: revert, foo, none; }",
    ":local(.foo) { animation-name: revert, :local(foo), none; }",
  );
}

#[test]
fn not_localize_revert_layer() {
  test(
    ".foo { animation: revert-layer; }",
    ":local(.foo) { animation: revert-layer; }",
  );
  test(
    ".foo { animation-name: revert-layer; }",
    ":local(.foo) { animation-name: revert-layer; }",
  );
}

#[test]
fn localize_animation_using_special_characters() {
  test(
    ".foo { animation: \\@bounce; }",
    ":local(.foo) { animation: :local(\\@bounce); }",
  );
  test(
    ".foo { animation: bou\\@nce; }",
    ":local(.foo) { animation: :local(bou\\@nce); }",
  );
  test(
    ".foo { animation: \\ as; }",
    ":local(.foo) { animation: :local(\\ as); }",
  );
  test(
    ".foo { animation: t\\ t; }",
    ":local(.foo) { animation: :local(t\\ t); }",
  );
  test(
    ".foo { animation: -\\a; }",
    ":local(.foo) { animation: :local(-\\a); }",
  );
  test(
    ".foo { animation: --\\a; }",
    ":local(.foo) { animation: :local(--\\a); }",
  );
  test(
    ".foo { animation: \\a; }",
    ":local(.foo) { animation: :local(\\a); }",
  );
  test(
    ".foo { animation: -\\a; }",
    ":local(.foo) { animation: :local(-\\a); }",
  );
  test(
    ".foo { animation: --; }",
    ":local(.foo) { animation: :local(--); }",
  );
  test(
    ".foo { animation: 😃bounce😃; }",
    ":local(.foo) { animation: :local(😃bounce😃); }",
  );
  test(
    ".foo { animation: --foo; }",
    ":local(.foo) { animation: :local(--foo); }",
  );
}

#[test]
fn not_localize_name_in_nested_function() {
  test(
    ".foo { animation: fade .2s var(--easeOutQuart) .1s forwards }",
    ":local(.foo) { animation: :local(fade) .2s var(--easeOutQuart) .1s forwards }",
  );
  test(
    ".foo { animation: fade .2s env(FOO_BAR) .1s forwards, name }",
    ":local(.foo) { animation: :local(fade) .2s env(FOO_BAR) .1s forwards, :local(name) }",
  );
  test(
    ".foo { animation: var(--foo-bar) .1s forwards, name }",
    ":local(.foo) { animation: var(--foo-bar) .1s forwards, :local(name) }",
  );
  test(
    ".foo { animation: var(--foo-bar) .1s forwards name, name }",
    ":local(.foo) { animation: var(--foo-bar) .1s forwards :local(name), :local(name) }",
  );
}

#[test]
fn localize_animation() {
  test(
    ".foo { animation: a; }",
    ":local(.foo) { animation: :local(a); }",
  );
  test(
    ".foo { animation: bar 5s, foobar; }",
    ":local(.foo) { animation: :local(bar) 5s, :local(foobar); }",
  );
  test(
    ".foo { animation: ease ease; }",
    ":local(.foo) { animation: ease :local(ease); }",
  );
  test(
    ".foo { animation: 0s ease 0s 1 normal none test running; }",
    ":local(.foo) { animation: 0s ease 0s 1 normal none :local(test) running; }",
  );
}

#[test]
fn localize_animation_with_vendor_prefix() {
  test(
    ".foo { -webkit-animation: bar; animation: bar; }",
    ":local(.foo) { -webkit-animation: :local(bar); animation: :local(bar); }",
  );
}

#[test]
fn not_localize_other_rules() {
  test(
    ".foo { content: \"animation: bar;\" }",
    ":local(.foo) { content: \"animation: bar;\" }",
  );
}

#[test]
fn not_localize_global_rules() {
  test(
    ":global .foo { animation: foo; animation-name: bar; }",
    ".foo { animation: foo; animation-name: bar; }",
  );
}

#[test]
fn handle_nested_global() {
  test(":global .a:not(:global .b) {}", ".a:not(.b) {}");
  test(
    ":global .a:not(:global .b:not(:global .c)) {}",
    ".a:not(.b:not(.c)) {}",
  );
  test(
    ":local .a:not(:not(:not(:global .c))) {}",
    ":local(.a):not(:not(:not(.c))) {}",
  );
  test(
    ":global .a:not(:global .b, :global .c) {}",
    ".a:not(.b, .c) {}",
  );
  test(
    ":local .a:not(:global .b, :local .c) {}",
    ":local(.a):not(.b, :local(.c)) {}",
  );
  test(
    ":global .a:not(:local .b, :global .c) {}",
    ".a:not(:local(.b), .c) {}",
  );
  test(":global .a:not(.b, .c) {}", ".a:not(.b, .c) {}");
  test(
    ":local .a:not(.b, .c) {}",
    ":local(.a):not(:local(.b), :local(.c)) {}",
  );
  test(
    ":global .a:not(:local .b, .c) {}",
    ".a:not(:local(.b), :local(.c)) {}",
  );
}

#[test]
fn handle_a_complex_animation_rule() {
  test(
    ".foo { animation: foo, bar 5s linear 2s infinite alternate, barfoo 1s; }",
    ":local(.foo) { animation: :local(foo), :local(bar) 5s linear 2s infinite alternate, :local(barfoo) 1s; }",
  );
}

#[test]
fn handle_animations_where_the_first_value_is_not_the_animation_name() {
  test(
    ".foo { animation: 1s foo; }",
    ":local(.foo) { animation: 1s :local(foo); }",
  );
}

#[test]
fn handle_animations_where_the_first_value_is_not_the_animation_name_whilst_also_using_keywords() {
  test(
    ".foo { animation: 1s normal ease-out infinite foo; }",
    ":local(.foo) { animation: 1s normal ease-out infinite :local(foo); }",
  );
}

#[test]
fn not_treat_animation_curve_as_identifier_of_animation_name_even_if_it_separated_by_comma() {
  test(
    ".foo { animation: slide-right 300ms forwards ease-out, fade-in 300ms forwards ease-out; }",
    ":local(.foo) { animation: :local(slide-right) 300ms forwards ease-out, :local(fade-in) 300ms forwards ease-out; }",
  );
}

#[test]
fn not_treat_start_and_end_keywords_in_steps_function_as_identifiers() {
  test(
    indoc! {r#"
            .foo { animation: spin 1s steps(12, end) infinite; }
            .foo { animation: spin 1s STEPS(12, start) infinite; }
            .foo { animation: spin 1s steps(12, END) infinite; }
            .foo { animation: spin 1s steps(12, START) infinite; }
        "#},
    indoc! {r#"
            :local(.foo) { animation: :local(spin) 1s steps(12, end) infinite; }
            :local(.foo) { animation: :local(spin) 1s STEPS(12, start) infinite; }
            :local(.foo) { animation: :local(spin) 1s steps(12, END) infinite; }
            :local(.foo) { animation: :local(spin) 1s steps(12, START) infinite; }
        "#},
  );
}

#[test]
fn handle_animations_with_custom_timing_functions() {
  test(
    ".foo { animation: 1s normal cubic-bezier(0.25, 0.5, 0.5. 0.75) foo; }",
    ":local(.foo) { animation: 1s normal cubic-bezier(0.25, 0.5, 0.5. 0.75) :local(foo); }",
  );
}

#[test]
fn handle_animations_whose_names_are_keywords() {
  test(
    ".foo { animation: 1s infinite infinite; }",
    ":local(.foo) { animation: 1s infinite :local(infinite); }",
  );
}

#[test]
fn handle_not_localize_an_animation_shorthand_value_of_inherit() {
  test(
    ".foo { animation: inherit; }",
    ":local(.foo) { animation: inherit; }",
  );
}

#[test]
fn handle_constructor_as_animation_name() {
  test(
    ".foo { animation: constructor constructor; }",
    // should be ":local(.foo) { animation: :local(constructor) :local(constructor); }"
    // but the output of postcss-modules-scope is "._local_foo) { animation: _local_constructor :local(constructor); }"
    // postcss-modules-scope only process the first :local in decl.value
    // therefore seems our output is more appropriate and it's fine to have different output here
    ":local(.foo) { animation: constructor :local(constructor); }",
  );
}

#[test]
fn default_to_global_when_mode_provided() {
  test_with_options(".foo {}", ".foo {}", LocalByDefault { mode: Mode::Global });
}

#[test]
fn default_to_local_when_mode_provided() {
  test_with_options(
    ".foo {}",
    ":local(.foo) {}",
    LocalByDefault { mode: Mode::Local },
  );
}

#[test]
fn use_correct_spacing() {
  test_with_options_warning(
    indoc! {r#"
            .a :local .b {}
            .a:local.b {}
            .a:local(.b) {}
            .a:local( .b ) {}
            .a :local(.b) {}
            .a :local( .b ) {}
            :local(.a).b {}
            :local( .a ).b {}
            :local(.a) .b {}
            :local( .a ) .b {}
        "#},
    indoc! {r#"
            .a :local(.b) {}
            .a:local(.b) {}
            .a:local(.b) {}
            .a:local(.b) {}
            .a :local(.b) {}
            .a :local(.b) {}
            :local(.a).b {}
            :local(.a).b {}
            :local(.a) .b {}
            :local(.a) .b {}
        "#},
    LocalByDefault { mode: Mode::Global },
    "Missing trailing whitespace",
  )
}

#[test]
fn localize_keyframes() {
  test(
    "@keyframes foo { from { color: red; } to { color: blue; } }",
    "@keyframes :local(foo) { from { color: red; } to { color: blue; } }",
  );
}

#[test]
fn localize_keyframes_starting_with_special_characters() {
  test(
    "@keyframes \\@foo { from { color: red; } to { color: blue; } }",
    "@keyframes :local(\\@foo) { from { color: red; } to { color: blue; } }",
  );
}

#[test]
fn localize_keyframes_containing_special_characters() {
  test(
    "@keyframes f\\@oo { from { color: red; } to { color: blue; } }",
    "@keyframes :local(f\\@oo) { from { color: red; } to { color: blue; } }",
  );
}

#[test]
fn localize_keyframes_in_global_default_mode() {
  test_with_options(
    "@keyframes foo {}",
    "@keyframes foo {}",
    LocalByDefault { mode: Mode::Global },
  );
}

#[test]
fn localize_explicit_keyframes() {
  test(
    "@keyframes :local(foo) { 0% { color: red; } 33.3% { color: yellow; } 100% { color: blue; } } @-webkit-keyframes :global(bar) { from { color: red; } to { color: blue; } }",
    "@keyframes :local(foo) { 0% { color: red; } 33.3% { color: yellow; } 100% { color: blue; } } @-webkit-keyframes bar { from { color: red; } to { color: blue; } }",
  );
}

#[test]
fn ignore_export_statements() {
  test(":export { foo: __foo; }", ":export { foo: __foo; }");
}

#[test]
fn ignore_import_statemtents() {
  test(
    ":import(\"~/lol.css\") { foo: __foo; }",
    ":import(\"~/lol.css\") { foo: __foo; }",
  );
}

#[test]
fn incorrectly_handle_nested_selectors() {
  test(
    ".bar:not(:global .foo, .baz) {}",
    ":local(.bar):not(.foo, .baz) {}",
  );
}

#[test]
fn compile_in_pure_mode() {
  test_with_options(
    ":global(.foo).bar, [type=\"radio\"] ~ .label, :not(.foo), #bar {}",
    ".foo:local(.bar), [type=\"radio\"] ~ :local(.label), :not(:local(.foo)), :local(#bar) {}",
    LocalByDefault { mode: Mode::Pure },
  );
}

#[test]
fn compile_explict_global_element() {
  test(":global(input) {}", "input {}");
}

#[test]
fn compile_explict_global_attribute() {
  test(
    ":global([type=\"radio\"]), :not(:global [type=\"radio\"]) {}",
    "[type=\"radio\"], :not([type=\"radio\"]) {}",
  );
}
#[test]
fn throw_on_inconsistent_selector_result() {
  test_with_warning(
    ":global .foo, .bar {}",
    ".foo, :local(.bar) {}",
    "Inconsistent",
  );
}

#[test]
fn throw_on_nested_locals() {
  test_with_warning(
    ":local(:local(.foo)) {}",
    ":local(.foo) {}",
    "is not allowed inside",
  );
}

#[test]
fn throw_on_nested_globals() {
  test_with_warning(
    ":global(:global(.foo)) {}",
    ".foo {}",
    "is not allowed inside",
  );
}

#[test]
fn throw_on_nested_mixed() {
  test_with_warning(
    ":local(:global(.foo)) {}",
    ".foo {}",
    "is not allowed inside",
  );
}

#[test]
fn throw_on_nested_broad_local() {
  test_with_warning(
    ":global(:local .foo) {}",
    ":local(.foo) {}",
    "is not allowed inside",
  );
}

#[test]
fn throw_on_incorrect_spacing_with_broad_global() {
  test_with_warning(
    ".foo :global.bar {}",
    ":local(.foo) .bar {}",
    "Missing trailing whitespace",
  );
}

#[test]
fn throw_on_incorrect_spacing_with_broad_local() {
  test(".foo:local .bar {}", ":local(.foo):local(.bar) {}");
}

#[test]
fn throw_on_not_pure_selector_global_class() {
  test_with_options_warning(
    ":global(.foo) {}",
    ".foo {}",
    LocalByDefault { mode: Mode::Pure },
    "Selector is not pure",
  );
}

#[test]
fn throw_on_not_pure_selector_with_multiple() {
  test_with_options_warning(
    ".foo, :global(.bar) {}",
    ":local(.foo), .bar {}",
    LocalByDefault { mode: Mode::Pure },
    "Selector is not pure",
  );
  test_with_options_warning(
    ":global(.bar), .foo {}",
    ".bar, :local(.foo) {}",
    LocalByDefault { mode: Mode::Pure },
    "Selector is not pure",
  );
}

#[test]
fn throw_on_not_pure_selector_element() {
  test_with_options_warning(
    "input {}",
    "input {}",
    LocalByDefault { mode: Mode::Pure },
    "Selector is not pure",
  );
  test_with_options_warning(
    "[type=\"radio\"] {}",
    "[type=\"radio\"] {}",
    LocalByDefault { mode: Mode::Pure },
    "Selector is not pure",
  );
}

#[test]
fn throw_on_not_pure_keyframes() {
  test_with_options_warning(
    "@keyframes :global(foo) {}",
    "@keyframes foo {}",
    LocalByDefault { mode: Mode::Pure },
    "'@keyframes :global' is not allowed in pure mode",
  );
}

#[test]
fn pass_through_global_element() {
  test("input {}", "input {}");
}

#[test]
fn localise_class_and_pass_through_element() {
  test(".foo input {}", ":local(.foo) input {}");
}

#[test]
fn pass_through_attribute_selector() {
  test("[type=\"radio\"] {}", "[type=\"radio\"] {}");
}

#[test]
fn not_modify_urls_without_option() {
  test(
    indoc! {r#"
            .a { background: url(./image.png); }
            :global .b { background: url(image.png); }
            .c { background: url("./image.png"); }
        "#},
    indoc! {r#"
            :local(.a) { background: url(./image.png); }
            .b { background: url(image.png); }
            :local(.c) { background: url("./image.png"); }
        "#},
  );
}

#[test]
fn rewrite_url_in_local_block() {
  test(
    indoc! {r#"
            .a { background: url(./image.png); }
            :global .b { background: url(image.png); }
            .c { background: url("./image.png"); }
            .c { background: url('./image.png'); }
            .d { background: -webkit-image-set(url("./image.png") 1x, url("./image2x.png") 2x); }
            @font-face { src: url("./font.woff"); }
            @-webkit-font-face { src: url("./font.woff"); }
            @media screen { .a { src: url("./image.png"); } }
            @keyframes :global(ani1) { 0% { src: url("image.png"); } }
            @keyframes ani2 { 0% { src: url("./image.png"); } }
            foo { background: end-with-url(something); }
        "#},
    indoc! {r#"
            :local(.a) { background: url(./image.png); }
            .b { background: url(image.png); }
            :local(.c) { background: url("./image.png"); }
            :local(.c) { background: url('./image.png'); }
            :local(.d) { background: -webkit-image-set(url("./image.png") 1x, url("./image2x.png") 2x); }
            @font-face { src: url("./font.woff"); }
            @-webkit-font-face { src: url("./font.woff"); }
            @media screen { :local(.a) { src: url("./image.png"); } }
            @keyframes ani1 { 0% { src: url("image.png"); } }
            @keyframes :local(ani2) { 0% { src: url("./image.png"); } }
            foo { background: end-with-url(something); }
        "#},
  );
}

#[test]
fn not_crash_on_atrule_without_nodes() {
  test("@charset \"utf-8\";", "@charset \"utf-8\";");
}

#[test]
fn not_crash_on_a_rule_without_nodes() {
  test(".a { .b {} }", ":local(.a) { :local(.b) {} }");
}

#[test]
fn not_break_unicode_characters() {
  test(
    r#".a { content: "\\2193" }"#,
    r#":local(.a) { content: "\\2193" }"#,
  );
  test(
    r#".a { content: "\\2193\\2193" }"#,
    r#":local(.a) { content: "\\2193\\2193" }"#,
  );
  test(
    r#".a { content: "\\2193 \\2193" }"#,
    r#":local(.a) { content: "\\2193 \\2193" }"#,
  );
  test(
    r#".a { content: "\\2193\\2193\\2193" }"#,
    r#":local(.a) { content: "\\2193\\2193\\2193" }"#,
  );
  test(
    r#".a { content: "\\2193 \\2193 \\2193" }"#,
    r#":local(.a) { content: "\\2193 \\2193 \\2193" }"#,
  );
}

#[test]
fn not_ignore_custom_property_set() {
  test(
    ":root { --title-align: center; --sr-only: { position: absolute; } }",
    ":root { --title-align: center; --sr-only: { position: absolute; } }",
  );
}

#[test]
fn not_localize_imported_alias() {
  test(
    indoc! {r#"
            :import(foo) { a_value: some-value; }

            .foo > .a_value { }
        "#},
    indoc! {r#"
            :import(foo) { a_value: some-value; }

            :local(.foo) > .a_value { }
        "#},
  );
}

#[test]
fn not_localize_nested_imported_alias() {
  test(
    indoc! {r#"
            :import(foo) { a_value: some-value; }

            .foo > .a_value > .bar { }
        "#},
    indoc! {r#"
            :import(foo) { a_value: some-value; }

            :local(.foo) > .a_value > :local(.bar) { }
        "#},
  );
}

#[test]
fn ignore_imported_in_explicit_local() {
  test(
    indoc! {r#"
            :import(foo) { a_value: some-value; }

            :local(.a_value) { }
        "#},
    indoc! {r#"
            :import(foo) { a_value: some-value; }

            :local(.a_value) { }
        "#},
  );
}

#[test]
fn escape_local_context_with_explict_global() {
  test(
    indoc! {r#"
            :import(foo) { a_value: some-value; }

            :local .foo :global(.a_value) .bar { }
        "#},
    indoc! {r#"
            :import(foo) { a_value: some-value; }

            :local(.foo) .a_value :local(.bar) { }
        "#},
  );
}

#[test]
fn respect_explicit_local() {
  test(
    indoc! {r#"
            :import(foo) { a_value: some-value; }

            .a_value :local .a_value .foo :global .a_value { }
        "#},
    indoc! {r#"
            :import(foo) { a_value: some-value; }

            .a_value :local(.a_value) :local(.foo) .a_value { }
        "#},
  );
}

#[test]
fn not_localize_imported_animation_name() {
  test(
    indoc! {r#"
            :import(file) { a_value: some-value; }

            .foo { animation-name: a_value; }
        "#},
    indoc! {r#"
            :import(file) { a_value: some-value; }

            :local(.foo) { animation-name: a_value; }
        "#},
  );
}

#[test]
fn throw_on_invalid_syntax_class_usage() {
  test_with_warning(". {}", ". {}", "Invalid class selector syntax");
}

#[test]
fn throw_on_invalid_syntax_id_usage() {
  test_with_warning("# {}", "# {}", "Invalid id selector syntax");
}

#[test]
fn throw_on_invalid_syntax_local_class_usage() {
  test_with_warning(":local(.) {}", ". {}", "Invalid class selector syntax");
}

#[test]
fn throw_on_invalid_syntax_local_id_usage() {
  test_with_warning(":local(#) {}", "# {}", "Invalid id selector syntax");
}

#[test]
fn throw_on_invalid_global_class_usage() {
  test_with_warning(":global(.) {}", ". {}", "Invalid class selector syntax");
  test_with_warning(":global(#) {}", "# {}", "Invalid id selector syntax");
  test_with_warning(
    ":global(.a:not(:global .b, :global .c)) {}",
    ".a:not(.b, .c) {}",
    "A ':global' is not allowed inside of a ':local()' or ':global()'",
  );
  test_with_warning(
    ":global() {}",
    " {}",
    "':global()' or ':local()' can't be empty",
  );
}

#[test]
fn consider_nesting_statements_as_pure() {
  test_with_options(
    ".foo { &:hover { a_value: some-value; } }",
    ":local(.foo) { &:hover { a_value: some-value; } }",
    LocalByDefault { mode: Mode::Pure },
  );
}

#[test]
fn consider_selector_nesting_statements_as_pure() {
  test_with_options(
    ".foo { html &:hover { a_value: some-value; } }",
    ":local(.foo) { html &:hover { a_value: some-value; } }",
    LocalByDefault { mode: Mode::Pure },
  );
  test_with_options(
    ".foo { &:global(.bar) { a_value: some-value; } }",
    ":local(.foo) { &.bar { a_value: some-value; } }",
    LocalByDefault { mode: Mode::Pure },
  );
}

#[test]
fn throw_on_nested_nesting_selectors_without_a_local_selector() {
  test_with_options_warning(
    ":global(.foo) { &:hover { a_value: some-value; } }",
    ".foo { &:hover { a_value: some-value; } }",
    LocalByDefault { mode: Mode::Pure },
    "Selector is not pure",
  );
}

#[test]
fn css_nesting() {
  test(
    indoc! {r#"
            .foo {
                &.class {
                    a_value: some-value;
                }

                @media screen and (min-width: 900px) {
                    b_value: some-value;

                    .bar {
                        c_value: some-value;
                    }

                    &.baz {
                        c_value: some-value;
                    }
                }
            }
        "#},
    indoc! {r#"
            :local(.foo) {
                &:local(.class) {
                    a_value: some-value;
                }

                @media screen and (min-width: 900px) {
                    b_value: some-value;

                    :local(.bar) {
                        c_value: some-value;
                    }

                    &:local(.baz) {
                        c_value: some-value;
                    }
                }
            }
        "#},
  );
  test(
    indoc! {r#"
            :local(.foo) {
                &:local(.class) {
                    a_value: some-value;
                }

                @media screen and (min-width: 900px) {
                    b_value: some-value;

                    :local(.bar) {
                        c_value: some-value;
                    }

                    &:local(.baz) {
                        c_value: some-value;
                    }
                }
            }
        "#},
    indoc! {r#"
            :local(.foo) {
                &:local(.class) {
                    a_value: some-value;
                }

                @media screen and (min-width: 900px) {
                    b_value: some-value;

                    :local(.bar) {
                        c_value: some-value;
                    }

                    &:local(.baz) {
                        c_value: some-value;
                    }
                }
            }
        "#},
  );
  test(
    indoc! {r#"
            :local(.foo) {
                &:local(.class) {
                    a_value: some-value;
                }

                @media screen and (min-width: 900px) {
                    b_value: some-value;

                    :local(.bar) {
                        c_value: some-value;
                    }

                    &:local(.baz) {
                        c_value: some-value;
                    }
                }
            }
        "#},
    indoc! {r#"
            :local(.foo) {
                &:local(.class) {
                    a_value: some-value;
                }

                @media screen and (min-width: 900px) {
                    b_value: some-value;

                    :local(.bar) {
                        c_value: some-value;
                    }

                    &:local(.baz) {
                        c_value: some-value;
                    }
                }
            }
        "#},
  );
  test_with_options(
    indoc! {r#"
            .foo {
                &.class {
                    a_value: some-value;
                }

                @media screen and (min-width: 900px) {
                    b_value: some-value;

                    .bar {
                        c_value: some-value;
                    }

                    &.baz {
                        c_value: some-value;
                    }
                }
            }
        "#},
    indoc! {r#"
            :local(.foo) {
                &:local(.class) {
                    a_value: some-value;
                }

                @media screen and (min-width: 900px) {
                    b_value: some-value;

                    :local(.bar) {
                        c_value: some-value;
                    }

                    &:local(.baz) {
                        c_value: some-value;
                    }
                }
            }
        "#},
    LocalByDefault { mode: Mode::Pure },
  );
}

// #[test]
// fn consider_import_statements_pure() {
//     test_with_options(
//         ":import(\"~/lol.css\") { foo: __foo; }",
//         ":import(\"~/lol.css\") { foo: __foo; }",
//         Options { mode: Mode::Pure },
//     );
// }

#[test]
fn consider_export_statements_pure() {
  test_with_options(
    ":export { foo: __foo; }",
    ":export { foo: __foo; }",
    LocalByDefault { mode: Mode::Pure },
  );
}

#[test]
fn handle_negative_animation_delay_in_animation_shorthand() {
  test(
    ".foo { animation: 1s -500ms; }",
    ":local(.foo) { animation: 1s -500ms; }",
  );
  test(
    ".foo { animation: 1s -500.0ms; }",
    ":local(.foo) { animation: 1s -500.0ms; }",
  );
  test(
    ".foo { animation: 1s -500.0ms -a_value; }",
    ":local(.foo) { animation: 1s -500.0ms :local(-a_value); }",
  );
}

#[test]
fn at_scope_at_rule() {
  test(
    indoc! {r#"
            .article-header {
                color: red;
            }

            .article-body {
                color: blue;
            }

            @scope      (.article-body)     to       (.article-header)        {
                .article-body {
                    border: 5px solid black;
                    background-color: goldenrod;
                }
            }

            @scope(.article-body)to(.article-header){
                .article-footer {
                    border: 5px solid black;
                }
            }

            @scope    (   .article-body   )    {
                img {
                    border: 5px solid black;
                    background-color: goldenrod;
                }
            }

            @scope {
                :scope {
                    color: red;
                }
            }
        "#},
    indoc! {r#"
            :local(.article-header) {
                color: red;
            }

            :local(.article-body) {
                color: blue;
            }

            @scope      (:local(.article-body))     to       (:local(.article-header))        {
                :local(.article-body) {
                    border: 5px solid black;
                    background-color: goldenrod;
                }
            }

            @scope(:local(.article-body))to(:local(.article-header)){
                :local(.article-footer) {
                    border: 5px solid black;
                }
            }

            @scope    (   :local(.article-body)   )    {
                img {
                    border: 5px solid black;
                    background-color: goldenrod;
                }
            }

            @scope {
                :scope {
                    color: red;
                }
            }
        "#},
  );
  test(
    indoc! {r#"
            @scope (.article-body) to (figure) {
                .article-footer {
                    border: 5px solid black;
                }
            }
        "#},
    indoc! {r#"
            @scope (:local(.article-body)) to (figure) {
                :local(.article-footer) {
                    border: 5px solid black;
                }
            }
        "#},
  );
  test(
    indoc! {r#"
            @scope (:local(.article-body)) to (:global(.class)) {
                .article-footer {
                    border: 5px solid black;
                }
                :local(.class-1) {
                    color: red;
                }
                :global(.class-2) {
                    color: blue;
                }
            }
        "#},
    indoc! {r#"
            @scope (:local(.article-body)) to (.class) {
                :local(.article-footer) {
                    border: 5px solid black;
                }
                :local(.class-1) {
                    color: red;
                }
                .class-2 {
                    color: blue;
                }
            }
        "#},
  );
  test_with_options(
    indoc! {r#"
            @scope (.article-header) to (.class) {
                .article-footer {
                    border: 5px solid black;
                }
                .class-1 {
                    color: red;
                }
                .class-2 {
                    color: blue;
                }
            }
        "#},
    indoc! {r#"
            @scope (:local(.article-header)) to (:local(.class)) {
                :local(.article-footer) {
                    border: 5px solid black;
                }
                :local(.class-1) {
                    color: red;
                }
                :local(.class-2) {
                    color: blue;
                }
            }
        "#},
    LocalByDefault { mode: Mode::Pure },
  );
  test(
    indoc! {r#"
            @scope (.article-header) to (.class) {
                .article-footer {
                    src: url("./font.woff");
                }
            }
        "#},
    indoc! {r#"
            @scope (:local(.article-header)) to (:local(.class)) {
                :local(.article-footer) {
                    src: url("./font.woff");
                }
            }
        "#},
  );
  test(
    indoc! {r#"
            .foo {
                @scope (.article-header) to (.class) {
                    :scope {
                        background: blue;
                    }

                    .bar {
                        color: red;
                    }
                }
            }
        "#},
    indoc! {r#"
            :local(.foo) {
                @scope (:local(.article-header)) to (:local(.class)) {
                    :scope {
                        background: blue;
                    }

                    :local(.bar) {
                        color: red;
                    }
                }
            }
        "#},
  );
  test_with_options(
    indoc! {r#"
            @scope (:global(.article-header).foo) to (:global(.class).bar) {
                .bar {
                    color: red;
                }
            }
        "#},
    indoc! {r#"
            @scope (.article-header:local(.foo)) to (.class:local(.bar)) {
                :local(.bar) {
                    color: red;
                }
            }
        "#},
    LocalByDefault { mode: Mode::Pure },
  );
}
