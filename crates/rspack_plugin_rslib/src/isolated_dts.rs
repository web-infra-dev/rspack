use std::{collections::VecDeque, sync::Arc};

use rspack_core::{
  Compilation, DependencyCategory, IsolatedDts, Resolve, ResolveOptionsWithDependencyType,
  ResolveResult, TsconfigOptions, TsconfigReferences, diagnostics::ModuleBuildError,
};
use rspack_error::{Diagnostic, Error, Result};
use rspack_javascript_compiler::JavaScriptCompiler;
use rspack_paths::{Utf8Path, Utf8PathBuf};
use rspack_util::{identifier::absolute_to_request, node_path::NodePath};
use rustc_hash::FxHashSet as HashSet;
use swc_core::{
  common::FileName,
  ecma::{
    ast::EsVersion,
    parser::{Syntax, TsSyntax},
  },
};

use crate::plugin::SwcEmitDtsOptions;

pub(crate) struct IsolatedDtsAsset {
  pub resource_path: String,
  pub code: String,
}

pub(crate) struct CompletedIsolatedDtsOutputs {
  pub assets: Vec<IsolatedDtsAsset>,
  pub diagnostics: Vec<Diagnostic>,
}

struct IsolatedDtsReferences {
  issuer_resource_path: String,
  references: Vec<String>,
}

pub(crate) async fn complete_isolated_dts_outputs(
  compilation: &mut Compilation,
  options: &SwcEmitDtsOptions,
  roots: Vec<IsolatedDts>,
  module_resources: Vec<Utf8PathBuf>,
) -> Result<CompletedIsolatedDtsOutputs> {
  if roots.is_empty() {
    return Ok(CompletedIsolatedDtsOutputs {
      assets: Vec::new(),
      diagnostics: Vec::new(),
    });
  }

  let mut outputs = Vec::with_capacity(roots.len());
  let mut diagnostics = Vec::new();
  let mut queue = VecDeque::with_capacity(roots.len());

  for IsolatedDts {
    resource_path,
    code,
    references,
  } in roots
  {
    let has_references = !references.is_empty();
    if has_references {
      queue.push_back(IsolatedDtsReferences {
        issuer_resource_path: resource_path.clone(),
        references,
      });
    }
    outputs.push(IsolatedDtsAsset {
      resource_path,
      code,
    });
  }

  if queue.is_empty() {
    return Ok(CompletedIsolatedDtsOutputs {
      assets: outputs,
      diagnostics,
    });
  }

  let compiler_root = compilation.options.context.as_path().to_path_buf();
  let root_dir = resolve_path(&compiler_root, &options.root_dir);
  let fs = compilation.input_filesystem.clone();
  let tsconfig = default_tsconfig_options(compilation, &compiler_root, &fs).await;
  let resolver = compilation
    .resolver_factory
    .get(ResolveOptionsWithDependencyType {
      resolve_options: Some(Box::new(type_resolve_options(tsconfig))),
      resolve_to_context: false,
      dependency_category: DependencyCategory::Esm,
    });
  let mut javascript_compiler = None;
  // Reference completion should only generate files that are absent from the
  // normal module graph; normal module builds already own their dts diagnostics.
  let mut completed_resources: HashSet<Utf8PathBuf> = module_resources.into_iter().collect();
  completed_resources.extend(
    outputs
      .iter()
      .map(|output| resolve_path(&compiler_root, &output.resource_path)),
  );

  while let Some(IsolatedDtsReferences {
    issuer_resource_path,
    references,
  }) = queue.pop_front()
  {
    let issuer = resolve_path(&compiler_root, &issuer_resource_path);
    let issuer_dir = issuer.parent().unwrap_or(compiler_root.as_path());

    for request in references {
      let (result, resolve_dependencies) = resolver
        .resolve_with_context(issuer_dir.as_std_path(), &request)
        .await;
      compilation.file_dependencies.extend(
        resolve_dependencies
          .file_dependencies
          .into_iter()
          .map(Into::into),
      );
      compilation.missing_dependencies.extend(
        resolve_dependencies
          .missing_dependencies
          .into_iter()
          .map(Into::into),
      );
      let Ok(ResolveResult::Resource(resource)) = result else {
        continue;
      };

      let resource_path = resource.path.node_normalize();
      let resource_path_str = resource_path.as_str();
      let resource_extension = resource_path.extension();
      let is_declaration_file = resource_path_str.ends_with(".d.ts")
        || resource_path_str.ends_with(".d.mts")
        || resource_path_str.ends_with(".d.cts");
      let is_supported_ts_source =
        !is_declaration_file && matches!(resource_extension, Some("ts" | "tsx" | "mts" | "cts"));
      if !is_supported_ts_source || !resource_path.starts_with(&root_dir) {
        continue;
      }

      if !completed_resources.insert(resource_path.clone()) {
        continue;
      }

      let diagnostic_file = Utf8PathBuf::from(
        absolute_to_request(compiler_root.as_str(), resource_path.as_str()).into_owned(),
      );
      let std_resource_path = resource_path.clone().into_std_path_buf();
      compilation
        .file_dependencies
        .insert(std_resource_path.clone().into());

      let source = fs.read_to_string(&resource_path).await?;
      // Declaration completion only needs to parse TypeScript sources for fast dts.
      // Keep this intentionally minimal instead of reusing swc-loader transform options.
      let syntax = Syntax::Typescript(TsSyntax {
        tsx: matches!(resource_extension, Some("tsx")),
        decorators: true,
        disallow_ambiguous_jsx_like: matches!(resource_extension, Some("mts" | "cts")),
        ..Default::default()
      });
      let dts_output = javascript_compiler
        .get_or_insert_with(JavaScriptCompiler::new)
        .emit_isolated_dts_from_source(
          source,
          Arc::new(FileName::Real(std_resource_path)),
          syntax,
          EsVersion::EsNext,
        )?;
      let mut dts_diagnostics = dts_output.diagnostics.into_iter();
      if let Some(first) = dts_diagnostics.next() {
        let mut source_error = Error::error(first);
        source_error.code = Some("RslibPlugin".into());

        let mut dts_error = Error::error("Failed to generate declaration files.".to_string());
        dts_error.code = Some("RslibPlugin".into());
        dts_error.source_error = Some(Box::new(source_error));
        let remaining = dts_diagnostics.collect::<Vec<_>>();
        if !remaining.is_empty() {
          dts_error.help = Some(remaining.join("\n"));
        }

        let mut diagnostic: Diagnostic = Error::from(ModuleBuildError::new(dts_error, None)).into();
        diagnostic.file = Some(diagnostic_file);
        diagnostics.push(diagnostic);
        continue;
      }

      let has_references = !dts_output.references.is_empty();
      let resource_path = resource_path.as_str().to_string();
      if has_references {
        queue.push_back(IsolatedDtsReferences {
          issuer_resource_path: resource_path.clone(),
          references: dts_output.references,
        });
      }
      outputs.push(IsolatedDtsAsset {
        resource_path,
        code: dts_output.code,
      });
    }
  }

  Ok(CompletedIsolatedDtsOutputs {
    assets: outputs,
    diagnostics,
  })
}

fn resolve_path(base: &Utf8Path, value: &str) -> Utf8PathBuf {
  let path = Utf8Path::new(value);
  if path.is_absolute() {
    path.to_path_buf().node_normalize()
  } else {
    base.node_join(path).node_normalize()
  }
}

async fn default_tsconfig_options(
  compilation: &mut Compilation,
  compiler_root: &Utf8Path,
  fs: &Arc<dyn rspack_fs::ReadableFileSystem>,
) -> Option<TsconfigOptions> {
  if compilation.options.resolve.tsconfig.is_some() {
    return None;
  }

  let tsconfig = compiler_root.node_join("tsconfig.json");
  match fs.metadata(&tsconfig).await {
    Ok(metadata) if metadata.is_file => {
      compilation
        .file_dependencies
        .insert(tsconfig.clone().into_std_path_buf().into());
      Some(TsconfigOptions {
        config_file: tsconfig,
        references: TsconfigReferences::Disabled,
      })
    }
    _ => {
      compilation
        .missing_dependencies
        .insert(tsconfig.into_std_path_buf().into());
      None
    }
  }
}

fn type_resolve_options(tsconfig: Option<TsconfigOptions>) -> Resolve {
  Resolve {
    extensions: Some(vec![
      ".ts".into(),
      ".tsx".into(),
      ".mts".into(),
      ".cts".into(),
      ".d.ts".into(),
      ".d.mts".into(),
      ".d.cts".into(),
      "...".into(),
    ]),
    extension_alias: Some(vec![
      (
        ".js".into(),
        vec![".ts".into(), ".tsx".into(), ".d.ts".into(), ".js".into()],
      ),
      (".jsx".into(), vec![".tsx".into(), ".jsx".into()]),
      (
        ".mjs".into(),
        vec![".mts".into(), ".d.mts".into(), ".mjs".into()],
      ),
      (
        ".cjs".into(),
        vec![".cts".into(), ".d.cts".into(), ".cjs".into()],
      ),
    ]),
    fully_specified: Some(false),
    tsconfig,
    ..Default::default()
  }
}
