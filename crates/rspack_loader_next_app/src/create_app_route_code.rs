use rspack_error::Result;
use rspack_paths::Utf8Path;
use rspack_util::{fx_hash::FxIndexMap, json_stringify};
use serde_json::json;

use crate::{
  create_absolute_path,
  is_metadata_route::is_metadata_route,
  load_entrypoint::load_next_js_template,
  util::{ensure_leading_slash, is_group_segment},
};

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

pub async fn create_app_route_code(
  name: &str,
  page: &str,
  page_path: &str,
  package_root: &Utf8Path,
  page_extensions: &[String],
  next_config_output: &Option<String>,
  app_dir: &str,
) -> Result<String> {
  // routePath is the path to the route handler file,
  // but could be aliased e.g. private-next-app-dir/favicon.ico
  let route_path = page_path;

  // This, when used with the resolver will give us the pathname to the built
  // route handler file.
  let mut resolved_page_path = create_absolute_path(app_dir, route_path);

  // If this is a metadata route, then we need to use the metadata loader for
  // the route to ensure that the route is generated.
  let (file_base_name, ext) = {
    let resolved_page_path = Utf8Path::new(&resolved_page_path);
    (
      resolved_page_path.file_stem().unwrap().to_string(),
      resolved_page_path.extension(),
    )
  };
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
  replacements.insert("VAR_DEFINITION_FILENAME", file_base_name);
  replacements.insert("VAR_DEFINITION_BUNDLE_PATH", bundle_path);
  replacements.insert("VAR_RESOLVED_PAGE_PATH", resolved_page_path);

  let mut injections = FxIndexMap::default();
  injections.insert("nextConfigOutput", json_stringify(next_config_output));

  let entrypoint = load_next_js_template(
    "app-route.js",
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
    &route
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
