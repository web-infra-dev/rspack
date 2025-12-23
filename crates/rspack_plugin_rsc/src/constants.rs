use once_cell::sync::Lazy;
use regex::Regex;

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

pub static REGEX_CSS: Lazy<Regex> = Lazy::new(|| {
  #[allow(clippy::unwrap_used)]
  Regex::new(r"\.(css|less|sass|scss|styl|stylus|pcss|postcss|sss)(?:$|\?)$").unwrap()
});
