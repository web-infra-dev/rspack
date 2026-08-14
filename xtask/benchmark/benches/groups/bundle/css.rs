use rspack::builder::{Builder, CompilerBuilder};
use rspack_core::{ModuleOptions, ModuleRule, ModuleRuleEffect, ModuleType, RuleSetCondition};
use rspack_regex::RspackRegex;

use crate::groups::bundle::util::{BuilderOptions, basic_compiler_builder};

pub fn compiler() -> CompilerBuilder {
  basic_compiler_builder(BuilderOptions {
    project: "css",
    entry: "./index.css",
    swc_loader: false,
    native_output_filesystem: false,
  })
}

pub fn modules_compiler() -> CompilerBuilder {
  let mut builder = compiler();
  builder.module(ModuleOptions::builder().rule(ModuleRule {
    test: Some(RuleSetCondition::Regexp(
      RspackRegex::new("(?:bootstrap|tailwind)\\.css$").unwrap(),
    )),
    effect: ModuleRuleEffect {
      r#type: Some(ModuleType::CssModule),
      ..Default::default()
    },
    ..Default::default()
  }));
  builder
}
