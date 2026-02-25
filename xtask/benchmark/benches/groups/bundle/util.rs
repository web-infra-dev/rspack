use std::{path::PathBuf, sync::Arc};

use rspack::builder::{Builder, CompilerBuilder};
use rspack_core::{
  Compiler, Experiments, Mode, ModuleOptions, ModuleRule, ModuleRuleEffect, ModuleRuleUse,
  ModuleRuleUseLoader, Resolve, RuleSetCondition,
};
use rspack_fs::{MemoryFileSystem, NativeFileSystem};
use rspack_regex::RspackRegex;
use serde_json::json;

// Because `CompilerBuilder` is not `Clone`
pub type CompilerBuilderGenerator = Arc<dyn Fn() -> CompilerBuilder + Send + Sync>;

pub struct BuilderOptions {
  pub project: &'static str,
  pub entry: &'static str,
}

pub fn basic_compiler_builder(options: BuilderOptions) -> CompilerBuilder {
  let mut builder = Compiler::builder();

  let dir = PathBuf::from(env!("CARGO_WORKSPACE_DIR"))
    .join(".bench/rspack-benchcases")
    .canonicalize()
    .unwrap()
    .join(options.project);

  builder
    .context(dir.to_string_lossy().to_string())
    .entry("main", options.entry)
    .module(ModuleOptions::builder().rule(ModuleRule {
      test: Some(RuleSetCondition::Regexp(
        RspackRegex::new("\\.(j|t)s(x)?$").unwrap(),
      )),
      effect: ModuleRuleEffect {
        r#use: ModuleRuleUse::Array(vec![ModuleRuleUseLoader {
        loader: "builtin:swc-loader".to_string(),
        options: Some(json!({
            "jsc": {
                "parser": {
                    "syntax": "typescript",
                    "tsx": true,
                },
                "transform": {
                    "react": {
                        "runtime": "automatic",
                    },
                }
            },
        }).to_string()),
      }]),
        ..Default::default()
      },
      ..Default::default()
    }))
    .cache(rspack_core::CacheOptions::Disabled)
    .resolve(Resolve {
      extensions: Some(vec!["...".to_string(), ".jsx".to_string()]),
      ..Default::default()
    })
    .experiments(Experiments::builder().css(true))
    .input_filesystem(Arc::new(NativeFileSystem::new(false)))
    .output_filesystem(Arc::new(MemoryFileSystem::default()))
    .enable_loader_swc();

  builder
}

pub fn derive_projects(
  projects: Vec<(&'static str, CompilerBuilderGenerator)>,
) -> Vec<(String, CompilerBuilderGenerator)> {
  projects
    .into_iter()
    .flat_map(|(name, builder)| {
      let mut projects = Vec::new();

      {
        let builder = builder.clone();
        projects.push((
          format!("{name}-development"),
          Arc::new(move || {
            let mut builder = builder();
            builder.mode(Mode::Development);
            builder
          }) as CompilerBuilderGenerator,
        ));
      }

      {
        let builder = builder.clone();
        projects.push((
          format!("{name}-production-sourcemap"),
          Arc::new(move || {
            let mut builder = builder();
            builder.mode(Mode::Production);
            builder.devtool(rspack::builder::Devtool::SourceMap);
            builder
          }),
        ));
      }

      projects
    })
    .collect()
}
