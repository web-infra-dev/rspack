use std::collections::HashMap;
use std::default::Default;
use std::path::Path;

#[derive(Clone, PartialEq, Debug)]
pub struct StyleSourceType {
  pub is_async: bool,
  pub ext: String,
  pub is_module: bool,
  pub file_name: String,
  pub file_path: String,
  pub source_content_map: Option<HashMap<String, String>>,
}

impl Default for StyleSourceType {
  fn default() -> Self {
    Self {
      is_async: false,
      ext: "".to_string(),
      is_module: false,
      file_name: "".to_string(),
      file_path: "".to_string(),
      source_content_map: None,
    }
  }
}

///
/// 判断是否是 css 资源
///
pub fn is_style_source(filepath: &str) -> Option<StyleSourceType> {
  let file = Path::new(filepath);
  let mut style_source: StyleSourceType = Default::default();
  if file.is_dir() {
    return None;
  }
  if let Some(filename) = file.file_name() {
    style_source.file_path = filepath.to_string();
    style_source.ext = file
      .extension()?
      .to_os_string()
      .into_string()
      .expect(&format!("get extension failed: {}", filepath));
    return if &style_source.ext == "css"
      || &style_source.ext == "less"
      || &style_source.ext == "scss"
      || &style_source.ext == "sass"
    {
      style_source.file_name = filename.to_os_string().into_string().unwrap();
      if style_source.file_name.contains(".module.css")
        || style_source.file_name.contains(".module.less")
        || style_source.file_name.contains(".module.scss")
        || style_source.file_name.contains(".module.sass")
      {
        style_source.is_module = true
      }
      Some(style_source)
    } else {
      None
    };
  }
  None
}
