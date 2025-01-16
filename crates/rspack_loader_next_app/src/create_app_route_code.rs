use std::path;

use rspack_paths::{Utf8Path, Utf8PathBuf};
use serde_json::json;

// use crate::config_shared::NextConfig;
// use crate::constants::WEBPACK_RESOURCE_QUERIES;
use crate::load_entrypoint::load_next_js_template;
// use crate::metadata::is_metadata_route;
// use crate::next_metadata_route_loader::get_filename_and_extension;
// use crate::normalizers::app::{AppBundlePathNormalizer, AppPathnameNormalizer};
// use crate::page_extensions_type::PageExtensions;

pub enum NextConfigOutput {
  Standalone,
  Export,
}

pub async fn create_app_route_code(
  name: &str,
  page: &str,
  page_path: &Utf8Path,
  package_root: &Utf8Path,
  resolve_app_route: impl Fn(
    &Utf8Path,
  ) -> std::pin::Pin<
    Box<dyn std::future::Future<Output = Option<Utf8PathBuf>>>,
  >,
  page_extensions: &[String],
  next_config_output: &Option<NextConfigOutput>,
) -> Result<String, Box<dyn std::error::Error>> {
  // routePath is the path to the route handler file,
  // but could be aliased e.g. private-next-app-dir/favicon.ico
  let route_path = page_path;

  // This, when used with the resolver will give us the pathname to the built
  // route handler file.
  let mut resolved_page_path = resolve_app_route(route_path).await.ok_or_else(|| {
    format!(
      "Invariant: could not resolve page path for {} at {}",
      name, route_path
    )
  })?;

  // If this is a metadata route, then we need to use the metadata loader for
  // the route to ensure that the route is generated.
  let file_base_name = resolved_page_path.file_stem().unwrap();
  let mut resolved_page_path = resolved_page_path.into_string();
  if is_metadata_route(name) && file_base_name != "route" {
    let ext = get_filename_and_extension(&resolved_page_path).ext;
    let is_dynamic_route_extension = page_extensions.contains(&ext);

    resolved_page_path = format!(
      "next-metadata-route-loader?{}!?{}",
      json!({
          "filePath": resolved_page_path,
          "isDynamicRouteExtension": if is_dynamic_route_extension { "1" } else { "0" },
      })
      .to_string(),
      WEBPACK_RESOURCE_QUERIES.metadataRoute
    );
  }

  let pathname = AppPathnameNormalizer::new().normalize(page);
  let bundle_path = AppBundlePathNormalizer::new().normalize(page);

  let entrypoint = load_next_js_template(
    "app-route",
    package_root,
    &json!({
        "VAR_USERLAND": resolved_page_path,
        "VAR_DEFINITION_PAGE": page,
        "VAR_DEFINITION_PATHNAME": pathname,
        "VAR_DEFINITION_FILENAME": file_base_name,
        "VAR_DEFINITION_BUNDLE_PATH": bundle_path,
        "VAR_RESOLVED_PAGE_PATH": resolved_page_path,
    }),
    &json!({
        "nextConfigOutput": serde_json::to_string(next_config_output)?,
    }),
    Default::default(),
  )
  .await?;

  Ok(entrypoint)
}
