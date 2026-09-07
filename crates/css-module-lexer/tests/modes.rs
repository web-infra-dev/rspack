use crate::support::*;

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
      .map(|warning| Lexer::slice_range(input, warning.range())
        .expect("test setup must produce the expected value"))
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
      .expect("test setup must produce the expected value")
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
      .expect("test setup must produce the expected value")
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
