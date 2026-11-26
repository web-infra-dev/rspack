use css_module_lexer::{ParseAs, collect_dependencies_as};

use crate::support::*;

#[test]
fn block_contents_bare_declarations_resolve_urls() {
  let input = "color: red; background: url(a.png)";
  let (stylesheet_deps, stylesheet_warnings) =
    collect_dependencies_as(input, Mode::Css, ParseAs::Stylesheet);
  assert!(stylesheet_deps.is_empty());
  assert!(stylesheet_warnings.is_empty());

  let (deps, warnings) = collect_dependencies_as(input, Mode::Css, ParseAs::BlockContents);
  assert!(warnings.is_empty());
  assert_eq!(deps.len(), 1);
  assert_url_dependency(
    input,
    &deps[0],
    "a.png",
    UrlRangeKind::Function,
    "url(a.png)",
  );
}

#[test]
fn block_contents_handles_urls_image_set_and_custom_properties() {
  let input = concat!(
    "color: red; background: url(./pixel.png); ",
    "background-image: image-set('./pixel2.png' 1x); ",
    "section: url('./pixel3.png'), url(./pixel4.png); ",
    "--quote: '\"'; --custom: {a:b}; --wrapper-shaped: a{color:red}; ",
    "background: url(./pixel5.png); ",
    "content: '}'; background: url(./pixel6.png)"
  );
  let (deps, warnings) = collect_dependencies_as(input, Mode::Css, ParseAs::BlockContents);
  assert!(warnings.is_empty());
  assert_eq!(deps.len(), 6);
  assert_url_dependency(
    input,
    &deps[0],
    "./pixel.png",
    UrlRangeKind::Function,
    "url(./pixel.png)",
  );
  assert_url_dependency(
    input,
    &deps[1],
    "./pixel2.png",
    UrlRangeKind::Function,
    "'./pixel2.png'",
  );
  assert_url_dependency(
    input,
    &deps[2],
    "./pixel3.png",
    UrlRangeKind::String,
    "'./pixel3.png'",
  );
  assert_url_dependency(
    input,
    &deps[3],
    "./pixel4.png",
    UrlRangeKind::Function,
    "url(./pixel4.png)",
  );
  assert_url_dependency(
    input,
    &deps[4],
    "./pixel5.png",
    UrlRangeKind::Function,
    "url(./pixel5.png)",
  );
  assert_url_dependency(
    input,
    &deps[5],
    "./pixel6.png",
    UrlRangeKind::Function,
    "url(./pixel6.png)",
  );
}

#[test]
fn block_contents_stray_right_curly_recovery() {
  // Stray `}` in block-item position starts a bad declaration running to `;` / EOF.
  let input = "background: url(a.png); } background: url(b.png); background: url(c.png)";
  let (deps, warnings) = collect_dependencies_as(input, Mode::Css, ParseAs::BlockContents);
  assert!(warnings.is_empty());
  assert_eq!(deps.len(), 2);
  assert_url_dependency(
    input,
    &deps[0],
    "a.png",
    UrlRangeKind::Function,
    "url(a.png)",
  );
  assert_url_dependency(
    input,
    &deps[1],
    "c.png",
    UrlRangeKind::Function,
    "url(c.png)",
  );

  // `color:red; }; color:blue`: `;` immediately closes the bad declaration after `}`.
  let input = "background: url(a.png); }; background: url(b.png)";
  let (deps, warnings) = collect_dependencies_as(input, Mode::Css, ParseAs::BlockContents);
  assert!(warnings.is_empty());
  assert_eq!(deps.len(), 2);
  assert_url_dependency(
    input,
    &deps[0],
    "a.png",
    UrlRangeKind::Function,
    "url(a.png)",
  );
  assert_url_dependency(
    input,
    &deps[1],
    "b.png",
    UrlRangeKind::Function,
    "url(b.png)",
  );

  // Without `;`, a stray `}` inside a declaration value does not end the declaration (CSS Syntax §5.4.7 nested=false).
  let input = "p: a url(a.png) } b url(b.png)";
  let (deps, warnings) = collect_dependencies_as(input, Mode::Css, ParseAs::BlockContents);
  assert!(warnings.is_empty());
  assert_eq!(deps.len(), 2);
  assert_url_dependency(
    input,
    &deps[0],
    "a.png",
    UrlRangeKind::Function,
    "url(a.png)",
  );
  assert_url_dependency(
    input,
    &deps[1],
    "b.png",
    UrlRangeKind::Function,
    "url(b.png)",
  );
}

#[test]
fn block_contents_unclosed_values_and_nested_rules() {
  let input = "top: anchor-size(--a1 width";
  let (deps, warnings) = collect_dependencies_as(input, Mode::Css, ParseAs::BlockContents);
  assert!(deps.is_empty());
  assert!(warnings.is_empty());

  let input = "content: 'a";
  let (deps, warnings) = collect_dependencies_as(input, Mode::Css, ParseAs::BlockContents);
  assert!(deps.is_empty());
  assert!(warnings.is_empty());

  let input = concat!(
    "a:hover { background: url(a.png); } ",
    "@media (min-width: 1px) { background: url(b.png); } ",
    "background: url(c.png)"
  );
  let (deps, warnings) = collect_dependencies_as(input, Mode::Css, ParseAs::BlockContents);
  assert!(warnings.is_empty());
  assert_eq!(deps.len(), 3);
  assert_url_dependency(
    input,
    &deps[0],
    "a.png",
    UrlRangeKind::Function,
    "url(a.png)",
  );
  assert_url_dependency(
    input,
    &deps[1],
    "b.png",
    UrlRangeKind::Function,
    "url(b.png)",
  );
  assert_url_dependency(
    input,
    &deps[2],
    "c.png",
    UrlRangeKind::Function,
    "url(c.png)",
  );

  // Inside a nested block, a `}` DOES close that block and returns to top-level block-contents.
  let input = "&.active { color: red } background: url(a.png) } background: url(b.png);";
  let (deps, warnings) = collect_dependencies_as(input, Mode::Local, ParseAs::BlockContents);
  assert!(warnings.is_empty());
  assert_local_class_dependency(input, &deps[0], ".active", false);
  assert_url_dependency(
    input,
    &deps[1],
    "a.png",
    UrlRangeKind::Function,
    "url(a.png)",
  );
  assert_url_dependency(
    input,
    &deps[2],
    "b.png",
    UrlRangeKind::Function,
    "url(b.png)",
  );
}

#[test]
fn block_contents_css_modules_scoping_and_pure_mode() {
  let input = concat!(
    "@keyframes spin { from { opacity: 0; } to { opacity: 1; } } ",
    "--brand: red; ",
    "color: var(--brand); ",
    "animation: spin 1s"
  );
  let (deps, warnings) = collect_dependencies_as(input, Mode::Local, ParseAs::BlockContents);
  assert!(warnings.is_empty());
  assert_local_keyframes_decl_dependency(input, &deps[0], "spin");
  assert_local_var_decl_dependency(input, &deps[1], "brand");
  assert_local_var_dependency(input, &deps[2], "brand", None);
  assert_local_keyframes_dependency(input, &deps[3], "spin");
  assert_eq!(deps.len(), 4);

  // `:global .foo {}` must restore default property mode for following top-level declarations.
  let input = ":global .foo { --g: 1px; } --local: 2px; color: var(--local);";
  let (deps, warnings) = collect_dependencies_as(input, Mode::Local, ParseAs::BlockContents);
  assert!(warnings.is_empty());
  assert_replace_dependency(input, &deps[0], "", ":global ");
  assert_local_var_decl_dependency(input, &deps[1], "local");
  assert_local_var_dependency(input, &deps[2], "local", None);
  assert_eq!(deps.len(), 3);

  // Commas in top-level declaration values must not emit NotPure warnings; rules are still checked.
  let input = "background: url(a.png), url(b.png); .btn { color: red; }";
  let (deps, warnings) = collect_dependencies_as(input, Mode::Pure, ParseAs::BlockContents);
  assert!(warnings.is_empty());
  assert_url_dependency(
    input,
    &deps[0],
    "a.png",
    UrlRangeKind::Function,
    "url(a.png)",
  );
  assert_url_dependency(
    input,
    &deps[1],
    "b.png",
    UrlRangeKind::Function,
    "url(b.png)",
  );
  assert_local_class_dependency(input, &deps[2], ".btn", false);
  assert_eq!(deps.len(), 3);

  let impure_input = "color: red, blue; div { color: green; }";
  let (_, warnings) = collect_dependencies_as(impure_input, Mode::Pure, ParseAs::BlockContents);
  assert_eq!(warnings.len(), 1);
  assert_warning(impure_input, &warnings[0], " div ");
}

#[test]
fn block_contents_imports_and_icss_at_top_level() {
  let input = indoc! {r#"
    @import "./dep.css";
    @value primary: #ff0000;
    :export { accent: primary; }
    color: primary;
  "#};
  let (deps, warnings) = collect_dependencies_as(input, Mode::Local, ParseAs::BlockContents);
  assert!(warnings.is_empty());
  assert_import_dependency(
    input,
    import_dependency(&deps, 0),
    "./dep.css",
    None,
    None,
    None,
    r#"@import "./dep.css";"#,
  );
  assert_icss_export_value_dependency(input, &deps[1], "primary", "#ff0000");
  assert_replace_dependency(input, &deps[2], "", "@value primary: #ff0000;");
  assert_icss_export_value_dependency(input, &deps[3], "accent", "primary");
  assert_replace_dependency(input, &deps[4], "", ":export { accent: primary; }");
  assert_icss_symbol_dependency(input, &deps[5], "primary", "primary");
  assert_eq!(deps.len(), 6);
}
