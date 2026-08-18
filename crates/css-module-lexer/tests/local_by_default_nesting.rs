use css_module_lexer::Mode;
use indoc::indoc;

use crate::local_by_default::*;

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
