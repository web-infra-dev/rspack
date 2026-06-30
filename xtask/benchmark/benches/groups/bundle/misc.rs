use rspack::builder::{Builder, CompilerBuilder};
use rspack_core::{
  ExportPresenceMode, JavascriptParserOptions, ModuleOptions, ModuleRule, ModuleRuleEffect,
  ModuleRuleUse, ModuleRuleUseLoader, ParserOptions, Resolve, RuleSetCondition,
};
use rspack_regex::RspackRegex;
use serde_json::json;

use crate::groups::bundle::util::{BuilderOptions, basic_compiler_builder};

pub fn compiler() -> CompilerBuilder {
  let mut builder = basic_compiler_builder(BuilderOptions {
    project: "misc",
    entry: "./src/index.ts",
    swc_loader: false,
    native_output_filesystem: false,
  });
  builder
    .target(vec!["node".to_string()])
    .resolve(Resolve {
      extensions: Some(vec![
        ".ts".to_string(),
        ".tsx".to_string(),
        "...".to_string(),
        ".jsx".to_string(),
      ]),
      ..Default::default()
    })
    .module(ModuleOptions::builder().rule(ModuleRule {
      test: Some(RuleSetCondition::Regexp(
        RspackRegex::new("\\.(j|t)s(x)?$").unwrap(),
      )),
      effect: ModuleRuleEffect {
        r#use: ModuleRuleUse::Array(vec![ModuleRuleUseLoader {
          loader: "builtin:swc-loader".to_string(),
          options: Some(
            json!({
                "jsc": {
                    "parser": {
                        "syntax": "typescript",
                        "tsx": true,
                    },
                    "transform": {
                        "react": {
                            "runtime": "classic",
                        },
                    }
                },
            })
            .to_string(),
          ),
        }]),
        parser: Some(ParserOptions::JavascriptAuto(JavascriptParserOptions {
          reexport_exports_presence: Some(ExportPresenceMode::None),
          ..Default::default()
        })),
        ..Default::default()
      },
      ..Default::default()
    }))
    .enable_loader_swc();
  builder
}
