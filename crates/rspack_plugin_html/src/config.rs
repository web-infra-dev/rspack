use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct HtmlPluginConfig {
  /// emitted file name in output path
  #[serde(default = "default_filename")]
  pub filename: String,
  /// template html file
  #[serde(default = "default_template")]
  pub template: String,
  /// `auto`, `head`, `body` or None
  pub inject: Option<String>,
  /// path or `auto`
  pub public_path: Option<String>,
  /// `blocking`, `defer`, or `module`
  #[serde(default = "default_script_loading")]
  pub script_loading: String,

  /// entry_chunk_name (only entry chunks are supported)
  pub chunks: Option<Vec<String>>,
  pub excluded_chunks: Option<Vec<String>>,

  /// hash func that used in subsource integrity
  /// sha384, sha256 or sha512
  pub sri: Option<String>,

  /// future
  pub minify: Option<bool>,
}

fn default_filename() -> String {
  String::from("index.html")
}

fn default_template() -> String {
  String::from("index.html")
}

fn default_script_loading() -> String {
  String::from("defer")
}

impl Default for HtmlPluginConfig {
  fn default() -> HtmlPluginConfig {
    HtmlPluginConfig {
      filename: default_filename(),
      template: default_template(),
      inject: None,
      public_path: None,
      script_loading: default_script_loading(),
      chunks: None,
      excluded_chunks: None,
      sri: None,
      minify: None,
    }
  }
}
