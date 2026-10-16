use std::{path::PathBuf, sync::Arc};

use criterion::Criterion;
use rspack::builder::{Builder, CompilerBuilder, Devtool};
use rspack_core::{
  AliasMap, CssModuleGeneratorOptions, GeneratorOptions, Mode, ModuleOptions, ModuleRule,
  ModuleRuleEffect, ModuleType, Resolve, RuleSetCondition,
};
use rspack_regex::RspackRegex;
use rspack_tasks::{CompilerContext, within_compiler_context, within_compiler_context_sync};

use crate::groups::{
  bundle::util::{BuilderOptions, basic_compiler_builder},
  diagnostics::assert_no_compilation_errors,
};

// Repeat complete, uncached builds within the measured region. CodSpeed simulation
// ignores Criterion's sample size, so changing sample_size alone would not help.
const BUILDS_PER_SAMPLE: usize = 10;

fn compiler() -> CompilerBuilder {
  let mut builder = basic_compiler_builder(BuilderOptions {
    project: "css/css-loader",
    entry: "./index.js",
    swc_loader: false,
    native_output_filesystem: false,
  });
  let fixtures =
    PathBuf::from(std::env::var("RSPACK_BENCHCASES_DIR").unwrap()).join("css/css-loader");
  // Adapt css-loader/rspack.config.mjs to the native Rust benchmark. foo.less
  // contains only CSS, so it can use the CSS parser without a JavaScript loader.
  let mut module = ModuleOptions::builder();
  module.rule(ModuleRule {
    test: Some(RuleSetCondition::Regexp(
      RspackRegex::new("\\.(css|less)$").unwrap(),
    )),
    effect: ModuleRuleEffect {
      r#type: Some(ModuleType::CssModule),
      ..Default::default()
    },
    ..Default::default()
  });

  let hash_name = "[name]--[local]--[fullhash]";
  for (index, (name, function, digest, length, salt)) in [
    (hash_name, None, Some("base64url"), Some(5), None),
    ("-1[local]", None, None, None, None),
    ("--[local]", None, None, None, None),
    ("__[local]", None, None, None, None),
    ("[local]--[fullhash]", None, None, None, None),
    ("😀- -[local]", None, None, None, None),
    (hash_name, Some("sha256"), None, Some(10), None),
    (hash_name, Some("xxhash64"), None, Some(6), None),
    (
      "prefix-[file][query]---[local]---[fullhash]-postfix",
      None,
      None,
      None,
      None,
    ),
    (hash_name, None, None, None, Some("my-custom-salt")),
    (hash_name, None, Some("hex"), Some(8), None),
    (hash_name, None, Some("base64"), Some(8), None),
    (hash_name, Some("md4"), Some("base64url"), Some(6), None),
    (
      hash_name,
      Some("sha256"),
      Some("hex"),
      Some(12),
      Some("another-salt"),
    ),
  ]
  .into_iter()
  .enumerate()
  {
    module.rule(ModuleRule {
      test: Some(RuleSetCondition::Regexp(
        RspackRegex::new("\\.css$").unwrap(),
      )),
      resource_query: Some(
        RuleSetCondition::Regexp(
          RspackRegex::new(&format!("\\?local-ident-name-{}$", index + 1)).unwrap(),
        )
        .into(),
      ),
      effect: ModuleRuleEffect {
        generator: Some(GeneratorOptions::CssModule(CssModuleGeneratorOptions {
          local_ident_name: Some(name.into()),
          local_ident_hash_function: function.map(Into::into),
          local_ident_hash_digest: digest.map(Into::into),
          local_ident_hash_digest_length: length,
          local_ident_hash_salt: salt.into(),
          ..Default::default()
        })),
        ..Default::default()
      },
      ..Default::default()
    });
  }
  builder
    .mode(Mode::Development)
    .devtool(Devtool::False)
    .target(vec!["web".into(), "es2022".into()])
    .module(module)
    .resolve(Resolve {
      alias: Some(
        vec![(
          "~test".into(),
          vec![AliasMap::Path(
            fixtures
              .join("node_modules/test")
              .to_string_lossy()
              .into_owned(),
          )],
        )]
        .into(),
      ),
      ..Default::default()
    });
  builder
}

pub(crate) fn benchmark(c: &mut Criterion) {
  let rt = rspack_benchmark::build_tokio_rt();
  let mut group = c.benchmark_group("bundle");
  group.bench_function("bundle@css-modules", |b| {
    b.iter_batched(
      || {
        (0..BUILDS_PER_SAMPLE)
          .map(|_| {
            let context = Arc::new(CompilerContext::new());
            let compiler =
              within_compiler_context_sync(context.clone(), || compiler().build().unwrap());
            (context, compiler)
          })
          .collect::<Vec<_>>()
      },
      |compilers| {
        for (context, mut compiler) in compilers {
          rt.block_on(within_compiler_context(context, async move {
            compiler.run().await.unwrap();
            assert_no_compilation_errors(&compiler.compilation, "css-modules benchmark build");
          }));
        }
      },
      criterion::BatchSize::PerIteration,
    );
  });
  group.finish();
}
