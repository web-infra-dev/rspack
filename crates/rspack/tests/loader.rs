#![allow(unused_imports)]

use rspack::builder::Builder as _;
use rspack_core::{
  Compiler, Experiments, ModuleOptions, ModuleRule, ModuleRuleEffect, ModuleRuleUse,
  ModuleRuleUseLoader, RuleSetCondition,
};
use rspack_paths::Utf8Path;
use rspack_regex::RspackRegex;
use serde_json::json;

#[cfg(feature = "loader_lightningcss")]
#[tokio::test(flavor = "multi_thread")]
async fn lightningcss() {
  let mut compiler = Compiler::builder()
    .context(Utf8Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/lightningcss"))
    .entry("main", "./src/index.js")
    .module(ModuleOptions::builder().rule(ModuleRule {
      test: Some(RuleSetCondition::Regexp(
        RspackRegex::new("\\.css$").unwrap(),
      )),
      effect: ModuleRuleEffect {
        r#use: ModuleRuleUse::Array(vec![ModuleRuleUseLoader {
          loader: "builtin:lightningcss-loader".to_string(),
          options: Some(json!({
            "include": 1 // lower nesting syntax
          }).to_string()),
        }]),
        ..Default::default()
      },
      ..Default::default()
    }))
    .experiments(Experiments::builder().css(true))
    .enable_loader_lightningcss()
    .build()
    .unwrap();

  compiler.build().await.unwrap();

  let errors: Vec<_> = compiler.compilation.get_errors().collect();
  assert!(errors.is_empty());
}

#[cfg(feature = "loader_swc")]
#[tokio::test(flavor = "multi_thread")]
async fn swc() {
  let mut compiler = Compiler::builder()
    .context(Utf8Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/swc"))
    .entry("main", "./src/index.jsx")
    .module(ModuleOptions::builder().rule(ModuleRule {
      test: Some(RuleSetCondition::Regexp(
        RspackRegex::new("\\.jsx$").unwrap(),
      )),
      effect: ModuleRuleEffect {
        r#use: ModuleRuleUse::Array(vec![ModuleRuleUseLoader {
          loader: "builtin:swc-loader".to_string(),
          options: Some(json!({
            "jsc": {
              "parser": {
                "syntax": "ecmascript",
                "jsx": true,
              },
              "transform": {
                "react": {
                  "runtime": "classic",
                  "pragma": "React.createElement",
                  "pragmaFrag": "React.Fragment",
                  "throwIfNamespace": true,
                  "useBuiltins": false
                }
              }
            }
          }).to_string()),
        }]),
        ..Default::default()
      },
      ..Default::default()
    }))
    .experiments(Experiments::builder().css(true))
    .enable_loader_swc()
    .build()
    .unwrap();

  compiler.build().await.unwrap();

  let errors: Vec<_> = compiler.compilation.get_errors().collect();
  assert!(errors.is_empty());
}

#[cfg(feature = "loader_react_refresh")]
#[tokio::test(flavor = "multi_thread")]
async fn react_refresh() {
  let mut compiler = Compiler::builder()
    .context(Utf8Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/react-refresh"))
    .entry("main", "./src/index.jsx")
    .module(ModuleOptions::builder().rule(ModuleRule {
      test: Some(RuleSetCondition::Regexp(
        RspackRegex::new("\\.jsx$").unwrap(),
      )),
      effect: ModuleRuleEffect {
        r#use: ModuleRuleUse::Array(vec![
          ModuleRuleUseLoader {
            loader: "builtin:react-refresh-loader".to_string(),
            options: None
          },
          ModuleRuleUseLoader {
          loader: "builtin:swc-loader".to_string(),
          options: Some(json!({
            "jsc": {
              "parser": {
                "syntax": "ecmascript",
                "jsx": true,
              },
              "transform": {
                "react": {
                  "runtime": "classic",
                  "pragma": "React.createElement",
                  "pragmaFrag": "React.Fragment",
                  "throwIfNamespace": true,
                  "useBuiltins": false
                }
              }
            }
          }).to_string()),
        }]),
        ..Default::default()
      },
      ..Default::default()
    }))
    .experiments(Experiments::builder().css(true))
    .enable_loader_swc()
    .enable_loader_react_refresh()
    .build()
    .unwrap();

  compiler.build().await.unwrap();

  let errors: Vec<_> = compiler.compilation.get_errors().collect();
  assert!(errors.is_empty());
}

#[cfg(feature = "loader_preact_refresh")]
#[tokio::test(flavor = "multi_thread")]
async fn preact_refresh() {
  let mut compiler = Compiler::builder()
    .context(Utf8Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/preact-refresh"))
    .entry("main", "./src/index.jsx")
    .module(ModuleOptions::builder().rule(ModuleRule {
      test: Some(RuleSetCondition::Regexp(
        RspackRegex::new("\\.jsx$").unwrap(),
      )),
      effect: ModuleRuleEffect {
        r#use: ModuleRuleUse::Array(vec![
          ModuleRuleUseLoader {
            loader: "builtin:preact-refresh-loader".to_string(),
            options: None
          },
          ModuleRuleUseLoader {
          loader: "builtin:swc-loader".to_string(),
          options: Some(json!({
            "jsc": {
              "parser": {
                "syntax": "ecmascript",
                "jsx": true,
              },
              "transform": {
                "react": {
                  "runtime": "classic",
                  "pragma": "Preact.h",
                  "pragmaFrag": "Preact.Fragment",
                  "throwIfNamespace": true,
                  "useBuiltins": false
                }
              }
            }
          }).to_string()),
        }]),
        ..Default::default()
      },
      ..Default::default()
    }))
    .experiments(Experiments::builder().css(true))
    .enable_loader_swc()
    .enable_loader_preact_refresh()
    .build()
    .unwrap();

  compiler.build().await.unwrap();

  let errors: Vec<_> = compiler.compilation.get_errors().collect();
  assert!(errors.is_empty());
}
