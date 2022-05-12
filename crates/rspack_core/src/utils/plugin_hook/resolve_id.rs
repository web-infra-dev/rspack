use crate::{plugin_driver::PluginDriver, BundleOptions, NormalizedBundleOptions, ResolvedURI};
use nodejs_resolver::{ResolveResult, Resolver};
use std::{ffi::OsString, path::Path, sync::Arc};
use sugar_path::PathSugar;
use tracing::instrument;

pub fn get_resolver(options: &NormalizedBundleOptions) -> Resolver {
  let resolver_option = &options.resolve;
  let resolver = Resolver::default()
    .with_extensions(
      resolver_option
        .extensions
        .iter()
        .map(|s| s.as_str())
        .collect(),
    )
    .with_alias(
      resolver_option
        .alias
        .iter()
        .map(|(s1, s2)| (s1.as_str(), s2.as_ref().map(|s| s.as_str())))
        .collect(),
    );
  resolver
}

#[inline]
pub fn is_external_module(source: &str) -> bool {
  source.starts_with("node:")
}

#[instrument(skip(plugin_driver))]
#[inline]
pub async fn resolve_id(
  source: &str,
  importer: Option<&str>,
  preserve_symlinks: bool,
  plugin_driver: &PluginDriver,
) -> ResolvedURI {
  let plugin_result = resolve_id_via_plugins(source, importer, plugin_driver).await;

  plugin_result.unwrap_or_else(|| {
    if importer.is_some() && is_external_module(source) {
      ResolvedURI::new(source.to_string(), true)
    } else {
      let id = if let Some(importer) = importer {
        let base_dir = Path::new(importer).parent().unwrap();
        let options = plugin_driver.ctx.as_ref().options.as_ref();
        let resolver = get_resolver(options);
        // Resolver::default().with_extensions(vec![".tsx", ".jsx", ".ts", ".js", ".json"]);
        match resolver.resolve(base_dir, source) {
          Ok(path) => match path {
            ResolveResult::Path(buf) => buf.to_string_lossy().to_string(),
            ResolveResult::Ignored => unreachable!(),
          },
          Err(reason) => panic!(
            "failed to load {} from {} due to  {}",
            &source, &importer, reason
          ),
        }
      } else {
        Path::new(source).resolve().to_string_lossy().to_string()
      };
      ResolvedURI::new(id, false)
    }
  })
}

#[inline]
pub async fn resolve_id_via_plugins(
  source: &str,
  importer: Option<&str>,
  plugin_driver: &PluginDriver,
) -> Option<ResolvedURI> {
  plugin_driver.resolve_id(source, importer).await
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
