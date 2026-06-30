use std::{path::PathBuf, sync::Arc};

use rspack::builder::{Builder, CompilerBuilder};
use rspack_core::{
  Compiler, Experiments, ExportPresenceMode, JavascriptParserOptions, Mode, ModuleOptions,
  ModuleRule, ModuleRuleEffect, ModuleRuleUse, ModuleRuleUseLoader, Optimization, OutputOptions,
  ParserOptions, Resolve, RuleSetCondition,
};
use rspack_fs::{MemoryFileSystem, NativeFileSystem, WritableFileSystem};
use rspack_regex::RspackRegex;
use serde_json::json;

// Because `CompilerBuilder` is not `Clone`
pub type CompilerBuilderGenerator = Arc<dyn Fn() -> CompilerBuilder + Send + Sync>;

pub struct BuilderOptions {
  pub project: &'static str,
  pub entry: &'static str,
  pub swc_loader: bool,
  pub native_output_filesystem: bool,
}

#[derive(Default)]
pub struct BuilderExtraOptions {
  pub swc_react_runtime: Option<&'static str>,
  pub resolve_extensions: Option<Vec<&'static str>>,
  pub ignore_missing_reexports: bool,
}

pub fn basic_compiler_builder(options: BuilderOptions) -> CompilerBuilder {
  basic_compiler_builder_with_extra_options(options, BuilderExtraOptions::default())
}

pub fn basic_compiler_builder_with_extra_options(
  options: BuilderOptions,
  extra_options: BuilderExtraOptions,
) -> CompilerBuilder {
  let mut builder = Compiler::builder();

  let benchcases_dir = std::env::var("RSPACK_BENCHCASES_DIR")
    .expect("RSPACK_BENCHCASES_DIR is required and must be an absolute path, e.g. RSPACK_BENCHCASES_DIR=/path/to/.bench/rspack-benchcases");
  let dir = PathBuf::from(benchcases_dir)
    .canonicalize()
    .unwrap()
    .join(options.project);

  let output_filesystem: Arc<dyn WritableFileSystem> = if options.native_output_filesystem {
    Arc::new(NativeFileSystem::new(false))
  } else {
    Arc::new(MemoryFileSystem::default())
  };
  let resolve_extensions = extra_options.resolve_extensions.map_or_else(
    || vec!["...".to_string(), ".jsx".to_string()],
    |extensions| extensions.into_iter().map(String::from).collect(),
  );

  builder
    .context(dir.to_string_lossy().to_string())
    .entry("main", options.entry)
    .cache(rspack_core::CacheOptions::Disabled)
    .optimization(Optimization::builder().minimize(false))
    .resolve(Resolve {
      extensions: Some(resolve_extensions),
      ..Default::default()
    })
    .experiments(Experiments::builder().css(true))
    .input_filesystem(Arc::new(NativeFileSystem::new(false)))
    .output_filesystem(output_filesystem);

  if options.native_output_filesystem {
    builder.output(OutputOptions::builder().compare_before_emit(false));
  }

  if options.swc_loader {
    let swc_react_runtime = extra_options.swc_react_runtime.unwrap_or("automatic");

    builder
      .module(ModuleOptions::builder().rule(ModuleRule {
        test: Some(RuleSetCondition::Regexp(
          RspackRegex::new("\\.(j|t)s(x)?$").unwrap(),
        )),
        effect:
          ModuleRuleEffect {
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
                                "runtime": swc_react_runtime,
                            },
                        }
                    },
                })
                .to_string(),
              ),
            }]),
            parser: extra_options.ignore_missing_reexports.then_some(
              ParserOptions::JavascriptAuto(JavascriptParserOptions {
                reexport_exports_presence: Some(ExportPresenceMode::None),
                ..Default::default()
              }),
            ),
            ..Default::default()
          },
        ..Default::default()
      }))
      .enable_loader_swc();
  }

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
