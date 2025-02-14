#![feature(trait_upcasting)]
#![allow(unused_attributes)]
#![allow(clippy::unwrap_used)]

use std::{path::PathBuf, sync::Arc};

use criterion::criterion_group;
use rspack::builder::Builder as _;
use rspack_benchmark::Criterion;
use rspack_core::{
  Compiler, Experiments, Mode, ModuleOptions, ModuleRule, ModuleRuleEffect, ModuleRuleUse,
  ModuleRuleUseLoader, Resolve, RuleSetCondition,
};
use rspack_fs::{MemoryFileSystem, NativeFileSystem};
use rspack_regex::RspackRegex;
use serde_json::json;
use tokio::runtime::Builder;

async fn basic_compile(production: bool) {
  let dir = std::env::var("CARGO_MANIFEST_DIR")
    .map(PathBuf::from)
    .or(
      // This is a workaround for the issue where the CARGO_MANIFEST_DIR is not set in the test environment
      std::env::var("CODSPEED_CARGO_WORKSPACE_ROOT")
        .map(|workspace_root| PathBuf::from(workspace_root).join("tasks/benchmark")),
    )
    .unwrap()
    .join("benches/modules_10000");

  let mut builder = Compiler::builder();
  builder
    .context(dir.to_string_lossy().to_string())
    .entry("main", "./index.jsx")
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
                },
                "externalHelpers": true,
            },
            "env": {
              "targets": "Chrome >= 48"
            }
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

  if production {
    builder.mode(Mode::Production);
  } else {
    builder.mode(Mode::Development);
  }

  let mut compiler = builder.build();

  compiler.run().await.unwrap();

  assert!(compiler
    .compilation
    .get_errors()
    .collect::<Vec<_>>()
    .is_empty());
}

pub fn modules_10000_benchmark(c: &mut Criterion) {
  let rt = Builder::new_multi_thread().build().unwrap();

  c.bench_function("10000_production", |b| {
    b.to_async(&rt).iter(|| basic_compile(true));
  });

  c.bench_function("10000_development", |b| {
    b.to_async(&rt).iter(|| basic_compile(false));
  });
}

criterion_group!(modules_10000, modules_10000_benchmark);
