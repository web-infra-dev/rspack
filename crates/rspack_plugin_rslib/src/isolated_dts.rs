use std::{collections::VecDeque, sync::Arc};

use rspack_core::{
  Compilation, DependencyCategory, IsolatedDts, Resolve, ResolveOptionsWithDependencyType,
  ResolveResult, TsconfigOptions, TsconfigReferences,
};
use rspack_error::{Result, error};
use rspack_javascript_compiler::JavaScriptCompiler;
use rspack_paths::{Utf8Path, Utf8PathBuf};
use rspack_util::node_path::NodePath;
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

struct IsolatedDtsReferences {
  issuer_resource_path: String,
  references: Vec<String>,
}

pub(crate) async fn complete_isolated_dts_outputs(
  compilation: &mut Compilation,
  options: &SwcEmitDtsOptions,
  roots: Vec<IsolatedDts>,
) -> Result<Vec<IsolatedDtsAsset>> {
  if roots.is_empty() {
    return Ok(Vec::new());
  }

  let mut outputs = Vec::with_capacity(roots.len());
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
    return Ok(outputs);
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
  let mut seen: HashSet<Utf8PathBuf> = outputs
    .iter()
    .map(|output| resolve_path(&compiler_root, &output.resource_path))
    .collect();

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
      compilation
        .file_dependencies
        .extend(resolve_dependencies.file_dependencies);
      compilation
        .missing_dependencies
        .extend(resolve_dependencies.missing_dependencies);
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

      if !seen.insert(resource_path.clone()) {
        continue;
      }

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
      handle_isolated_dts_diagnostics(&resource_path, dts_output.diagnostics)?;

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

  Ok(outputs)
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

fn handle_isolated_dts_diagnostics(
  resource_path: &Utf8Path,
  diagnostics: Vec<String>,
) -> Result<()> {
  let mut diagnostics = diagnostics.into_iter();
  let Some(first) = diagnostics.next() else {
    return Ok(());
  };
  let remaining = diagnostics.collect::<Vec<_>>();
  let help = if remaining.is_empty() {
    String::new()
  } else {
    format!("\n{}", remaining.join("\n"))
  };

  Err(error!(
    "Failed to generate declaration files for {}.\n{}{}",
    resource_path, first, help
  ))
}
