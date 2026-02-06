use std::sync::LazyLock;

use once_cell::sync::Lazy;
use regex::Regex;

/// The names of the Rspack layers. These layers are the primitives for the
/// Rspack chunks.
pub(crate) const LAYERS_NAMES: LayersNames = LayersNames {
  react_server_components: "react-server-components",
  server_side_rendering: "server-side-rendering",
};

pub(crate) struct LayersNames {
  pub react_server_components: &'static str,
  pub server_side_rendering: &'static str,
}

pub(crate) static CSS_REGEX: Lazy<Regex> = Lazy::new(|| {
  #[allow(clippy::unwrap_used)]
  Regex::new(r"\.(css|less|sass|scss|styl|stylus|pcss|postcss|sss)(?:$|\?)$").unwrap()
});

pub(crate) static IMAGE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
  let image_extensions = ["jpg", "jpeg", "png", "webp", "avif", "ico", "svg"];
  #[allow(clippy::unwrap_used)]
  Regex::new(&format!(r"\.({})$", image_extensions.join("|"))).unwrap()
});
