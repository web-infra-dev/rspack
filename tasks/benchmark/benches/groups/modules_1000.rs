#![feature(trait_upcasting)]
#![allow(unused_attributes)]
#![allow(clippy::unwrap_used)]

use std::{path::PathBuf, sync::Arc};

use criterion::criterion_group;
use rspack::builder::{Builder as _, OptimizationOptionsBuilder};
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
  let dir = PathBuf::from(env!("CARGO_WORKSPACE_DIR"))
    .join(".bench/rspack-benchcases")
    .canonicalize()
    .unwrap()
    .join("1000");

  println!("{:?}", &dir);

  let mut builder = Compiler::builder();

  println!("Builder init");
  builder
    .context(dir.to_string_lossy().to_string())
    .entry("main", "./src/index.jsx")
    .optimization(OptimizationOptionsBuilder::default().minimize(false))
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
  println!("Builder configured");

  if production {
    builder.mode(Mode::Production);
  } else {
    builder.mode(Mode::Development);
  }

  println!("Builder mode set");
  let mut compiler = builder.build().unwrap();

  println!("Compiler before run");
  compiler.run().await.unwrap();
  println!(
    "{:?}",
    compiler.compilation.get_errors().collect::<Vec<_>>()
  );
  assert!(compiler
    .compilation
    .get_errors()
    .collect::<Vec<_>>()
    .is_empty());
}

pub fn modules_1000_benchmark(c: &mut Criterion) {
  // Codspeed can only handle to up to 500 threads by default
  let rt = Builder::new_multi_thread()
    .max_blocking_threads(256)
    .build()
    .unwrap();

  c.bench_function("1000_production", |b| {
    b.to_async(&rt).iter(|| basic_compile(true));
  });

  c.bench_function("1000_development", |b| {
    b.to_async(&rt).iter(|| basic_compile(false));
  });
}

criterion_group!(modules_1000, modules_1000_benchmark);
