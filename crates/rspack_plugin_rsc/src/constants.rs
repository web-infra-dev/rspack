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
pub const WEBPACK_LAYERS_NAMES: WebpackLayersNames = WebpackLayersNames {
  shared: "shared",
  react_server_components: "rsc",
  server_side_rendering: "ssr",
  action_browser: "action-browser",
  api: "api",
  middleware: "middleware",
  instrument: "instrument",
  edge_asset: "edge-asset",
  app_pages_browser: "app-pages-browser",
};

pub struct WebpackLayersNames {
  pub shared: &'static str,
  pub react_server_components: &'static str,
  pub server_side_rendering: &'static str,
  pub action_browser: &'static str,
  pub api: &'static str,
  pub middleware: &'static str,
  pub instrument: &'static str,
  pub edge_asset: &'static str,
  pub app_pages_browser: &'static str,
}

pub type WebpackLayerName = &'static str;

pub const WEBPACK_LAYERS: WebpackLayers = WebpackLayers {
  shared: WEBPACK_LAYERS_NAMES.shared,
  react_server_components: WEBPACK_LAYERS_NAMES.react_server_components,
  server_side_rendering: WEBPACK_LAYERS_NAMES.server_side_rendering,
  action_browser: WEBPACK_LAYERS_NAMES.action_browser,
  api: WEBPACK_LAYERS_NAMES.api,
  middleware: WEBPACK_LAYERS_NAMES.middleware,
  instrument: WEBPACK_LAYERS_NAMES.instrument,
  edge_asset: WEBPACK_LAYERS_NAMES.edge_asset,
  app_pages_browser: WEBPACK_LAYERS_NAMES.app_pages_browser,
  group: WebpackLayersGroup {
    builtin_react: [
      WEBPACK_LAYERS_NAMES.react_server_components,
      WEBPACK_LAYERS_NAMES.action_browser,
    ],
    server_only: [
      WEBPACK_LAYERS_NAMES.react_server_components,
      WEBPACK_LAYERS_NAMES.action_browser,
      WEBPACK_LAYERS_NAMES.instrument,
      WEBPACK_LAYERS_NAMES.middleware,
    ],
    neutral_target: [WEBPACK_LAYERS_NAMES.api],
    client_only: [
      WEBPACK_LAYERS_NAMES.server_side_rendering,
      WEBPACK_LAYERS_NAMES.app_pages_browser,
    ],
    bundled: [
      WEBPACK_LAYERS_NAMES.react_server_components,
      WEBPACK_LAYERS_NAMES.action_browser,
      WEBPACK_LAYERS_NAMES.server_side_rendering,
      WEBPACK_LAYERS_NAMES.app_pages_browser,
      WEBPACK_LAYERS_NAMES.shared,
      WEBPACK_LAYERS_NAMES.instrument,
    ],
    app_pages: [
      WEBPACK_LAYERS_NAMES.react_server_components,
      WEBPACK_LAYERS_NAMES.server_side_rendering,
      WEBPACK_LAYERS_NAMES.app_pages_browser,
      WEBPACK_LAYERS_NAMES.action_browser,
    ],
  },
};

pub struct WebpackLayers {
  pub shared: WebpackLayerName,
  pub react_server_components: WebpackLayerName,
  pub server_side_rendering: WebpackLayerName,
  pub action_browser: WebpackLayerName,
  pub api: WebpackLayerName,
  pub middleware: WebpackLayerName,
  pub instrument: WebpackLayerName,
  pub edge_asset: WebpackLayerName,
  pub app_pages_browser: WebpackLayerName,
  pub group: WebpackLayersGroup,
}

pub struct WebpackLayersGroup {
  pub builtin_react: [WebpackLayerName; 2],
  pub server_only: [WebpackLayerName; 4],
  pub neutral_target: [WebpackLayerName; 1],
  pub client_only: [WebpackLayerName; 2],
  pub bundled: [WebpackLayerName; 6],
  pub app_pages: [WebpackLayerName; 4],
}

pub const APP_CLIENT_INTERNALS: &'static str = "app-pages-internals";

// server/server-reference-manifest
pub const SERVER_REFERENCE_MANIFEST: &str = "server-reference-manifest";

pub static REGEX_CSS: Lazy<Regex> = Lazy::new(|| Regex::new(r"\.(css|scss|sass)(\?.*)?$").unwrap());
