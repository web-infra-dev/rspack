use std::{path::PathBuf, sync::Arc};

use criterion::{criterion_group, Criterion};
use rspack::builder::{Builder, CompilerBuilder};
use rspack_core::{
  Compiler, Experiments, Mode, ModuleOptions, ModuleRule, ModuleRuleEffect, ModuleRuleUse,
  ModuleRuleUseLoader, Resolve, RuleSetCondition,
};
use rspack_fs::{MemoryFileSystem, NativeFileSystem};
use rspack_regex::RspackRegex;
use serde_json::json;
use tokio::runtime;

pub mod modules_1000;
pub mod threejs;

criterion_group!(bundle, bundle_benchmark);

fn bundle_benchmark(c: &mut Criterion) {
  let mut group = c.benchmark_group("bundle");

  #[cfg(feature = "codspeed")]
  group.sample_size(10);

  let projects: &[(&str, Box<dyn Fn() -> Compiler>)] = &[
    ("1000_production", Box::new(|| modules_1000::compiler(true))),
    (
      "1000_development",
      Box::new(|| modules_1000::compiler(false)),
    ),
    ("threejs_production", Box::new(|| threejs::compiler(true))),
    ("threejs_development", Box::new(|| threejs::compiler(false))),
  ];

  // Codspeed can only handle to up to 500 threads by default
  let rt = runtime::Builder::new_multi_thread()
    .max_blocking_threads(256)
    .build()
    .unwrap();

  for (id, get_compiler) in projects {
    group.bench_function(&format!("bundle@{id}"), |b| {
      b.to_async(&rt).iter(|| async {
        let mut compiler = get_compiler();
        compiler.build().await.unwrap();
      });
    });
  }
}

struct BuilderOptions {
  project: &'static str,
  entry: &'static str,
  is_production: bool,
}

fn basic_compiler_builder(options: BuilderOptions) -> CompilerBuilder {
  let mut builder = Compiler::builder();

  let dir = PathBuf::from(env!("CARGO_WORKSPACE_DIR"))
    .join(".bench/rspack-benchcases")
    .canonicalize()
    .unwrap()
    .join(&options.project);

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

  if options.is_production {
    builder.mode(Mode::Production);
  } else {
    builder.mode(Mode::Development);
  }

  builder
}
