use crate::support::*;

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
