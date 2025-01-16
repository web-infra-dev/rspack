use std::borrow::Cow;

use rspack_paths::{Utf8Path, Utf8PathBuf};
use rspack_util::fx_hash::FxIndexMap;
use serde_json::json;

use crate::{is_metadata_route::is_metadata_route, load_entrypoint::load_next_js_template};

pub struct WebpackResourceQueries {
  pub edge_ssr_entry: &'static str,
  pub metadata: &'static str,
  pub metadata_route: &'static str,
  pub metadata_image_meta: &'static str,
}

pub const WEBPACK_RESOURCE_QUERIES: WebpackResourceQueries = WebpackResourceQueries {
  edge_ssr_entry: "__next_edge_ssr_entry__",
  metadata: "__next_metadata__",
  metadata_route: "__next_metadata_route__",
  metadata_image_meta: "__next_metadata_image_meta__",
};

pub enum NextConfigOutput {
  Standalone,
  Export,
}

impl NextConfigOutput {
  pub fn as_str(&self) -> &'static str {
    match self {
      Self::Standalone => "standalone",
      Self::Export => "export",
    }
  }
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
  next_config_output: &NextConfigOutput,
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
  let ext = resolved_page_path.extension();
  let mut resolved_page_path = resolved_page_path.to_string();
  if is_metadata_route(name) && file_base_name != "route" {
    let is_dynamic_route_extension =
      ext.is_some_and(|ext| page_extensions.iter().any(|the_ext| ext == the_ext));

    resolved_page_path = format!(
      "next-metadata-route-loader?{}!?{}",
      json!({
          "filePath": resolved_page_path,
          "isDynamicRouteExtension": if is_dynamic_route_extension { "1" } else { "0" },
      })
      .to_string(),
      WEBPACK_RESOURCE_QUERIES.metadata_route
    );
  }

  let pathname = normalize_app_path(page);
  let bundle_path = normalize_app_path(page);

  let mut replacements = FxIndexMap::default();
  replacements.insert("VAR_USERLAND", resolved_page_path.to_string());
  replacements.insert("VAR_DEFINITION_PAGE", page.to_string());
  replacements.insert("VAR_DEFINITION_PATHNAME", pathname);
  replacements.insert("VAR_DEFINITION_FILENAME", file_base_name.to_string());
  replacements.insert("VAR_DEFINITION_BUNDLE_PATH", bundle_path);
  replacements.insert("VAR_RESOLVED_PAGE_PATH", resolved_page_path);

  let mut injections = FxIndexMap::default();
  injections.insert(
    "nextConfigOutput",
    serde_json::to_string(next_config_output.as_str())?,
  );

  let entrypoint = load_next_js_template(
    "app-route",
    package_root,
    replacements,
    injections,
    Default::default(),
  )
  .await?;

  Ok(entrypoint)
}

fn normalize_app_path(route: &str) -> String {
  let path = ensure_leading_slash(
    route
      .split('/')
      .filter(|segment| {
        !segment.is_empty() && !is_group_segment(segment) && !segment.starts_with('@')
      })
      .enumerate()
      .fold(String::new(), |mut pathname, (index, segment)| {
        if (segment == "page" || segment == "route") && index == route.split('/').count() - 1 {
          return pathname;
        }
        pathname.push('/');
        pathname.push_str(segment);
        pathname
      }),
  );
  path.replace("%5F", "_")
}

fn ensure_leading_slash(path: String) -> String {
  if path.starts_with('/') {
    path
  } else {
    format!("/{}", path)
  }
}

fn is_group_segment(segment: &str) -> bool {
  // Implement the logic to check if a segment is a group segment
  // This is a placeholder implementation
  segment.starts_with('(') && segment.ends_with(')')
}
