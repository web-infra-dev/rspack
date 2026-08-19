use css_module_lexer::Mode;
use indoc::indoc;

use crate::local_by_default::*;

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
