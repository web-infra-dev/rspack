use once_cell::sync::Lazy;
use regex::Regex;

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

pub const BARREL_OPTIMIZATION_PREFIX: &'static str = "__barrel_optimize__";

pub const UNDERSCORE_NOT_FOUND_ROUTE: &str = "/_not-found";
pub const UNDERSCORE_NOT_FOUND_ROUTE_ENTRY: &str = "/_not-found/page";

// pub static REGEX_CSS: Lazy<Regex> = Lazy::new(|| Regex::new(r"\.(css|scss|sass)(\?.*)?$").unwrap());

/// The names of the webpack layers. These layers are the primitives for the
/// webpack chunks.
pub const LAYERS_NAMES: LayersNames = LayersNames {
  react_server_components: "react-server-components",
  server_side_rendering: "server-side-rendering",
  action_browser: "action-browser",
};

pub struct LayersNames {
  pub react_server_components: &'static str,
  pub server_side_rendering: &'static str,
  pub action_browser: &'static str,
}

pub const SERVER_REFERENCE_MANIFEST: &str = "server-reference-manifest";

pub static REGEX_CSS: Lazy<Regex> = Lazy::new(|| Regex::new(r"\.(css|scss|sass)(\?.*)?$").unwrap());
