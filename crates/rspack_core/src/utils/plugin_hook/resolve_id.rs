use crate::{
  plugin_driver::PluginDriver, BundleOptions, NormalizedBundleOptions, OnResolveResult,
  ResolveArgs, ResolvedURI,
};
use nodejs_resolver::{ResolveResult, Resolver, ResolverOptions};
use once_cell::sync::OnceCell;
use std::{collections::HashMap, ffi::OsString, path::Path, sync::Arc, time::Instant};
use sugar_path::PathSugar;
use tracing::instrument;

#[inline]
pub fn is_external_module(source: &str) -> bool {
  source.starts_with("node:")
}

#[instrument(skip(plugin_driver))]
#[inline]
pub async fn resolve_id(
  args: ResolveArgs,
  preserve_symlinks: bool,
  plugin_driver: &PluginDriver,
  resolver: &Resolver,
) -> ResolvedURI {
  if let Some(plugin_result) = resolve_id_via_plugins(&args, plugin_driver).await {
    ResolvedURI::new(plugin_result.uri, plugin_result.external, args.kind.clone())
  } else {
    if args.importer.is_some() && is_external_module(&args.id) {
      ResolvedURI::new(args.id.to_string(), true, args.kind.clone())
    } else {
      let id = if let Some(importer) = &args.importer {
        let base_dir = Path::new(&importer).parent().unwrap();
        let options = plugin_driver.ctx.as_ref().options.as_ref();
        let before_resolve = Instant::now();
        let res = match resolver.resolve(base_dir, &args.id) {
          Ok(path) => match path {
            ResolveResult::Path(buf) => buf.to_string_lossy().to_string(),
            ResolveResult::Ignored => unreachable!(),
          },
          Err(reason) => panic!(
            "failed to resolve {} from {} due to  {}",
            &args.id, &importer, reason
          ),
        };
        let after_resolve = Instant::now();
        let diff = after_resolve.duration_since(before_resolve);
        if diff.as_millis() >= 100 {
          tracing::debug!(
            "resolve is slow({:?}ms) for base_dir: {:?}, source: {:?}",
            diff.as_millis(),
            base_dir,
            args.id
          );
        }
        res
      } else {
        Path::new(&args.id).resolve().to_string_lossy().to_string()
      };
      ResolvedURI::new(id, false, args.kind.clone())
    }
  }
}

#[inline]
pub async fn resolve_id_via_plugins(
  args: &ResolveArgs,
  plugin_driver: &PluginDriver,
) -> Option<OnResolveResult> {
  plugin_driver.resolve_id(args).await
}

#[inline]
pub fn fast_add_js_extension_if_necessary(mut file: String, _preserve_symlinks: bool) -> String {
  if !file.ends_with(".js") {
    file.push_str(".js");
  }
  file
}

pub fn add_js_extension_if_necessary(file: &str, preserve_symlinks: bool) -> String {
  let found = find_file(Path::new(file), preserve_symlinks);
  found.unwrap_or_else(|| {
    let found = find_file(Path::new(&(file.to_string() + "#.mjs")), preserve_symlinks);
    found.unwrap_or_else(|| {
      let found = find_file(Path::new(&(file.to_string() + ".js")), preserve_symlinks);
      found.unwrap()
    })
  })
}

pub fn find_file(file: &Path, preserve_symlinks: bool) -> Option<String> {
  let metadata = std::fs::metadata(file);
  if let Ok(metadata) = metadata {
    if !preserve_symlinks && metadata.is_symlink() {
      find_file(&std::fs::canonicalize(file).ok()?, preserve_symlinks)
    } else if (preserve_symlinks && metadata.is_symlink()) || metadata.is_file() {
      let name: OsString = file.file_name().unwrap().to_os_string();
      let files = std::fs::read_dir(file.parent().unwrap()).unwrap();

      files
        .map(|result| result.unwrap())
        .find(|file| file.file_name() == name)
        .map(|_| file.to_string_lossy().to_string())
    } else {
      None
    }
  } else {
    None
  }
}
