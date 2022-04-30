use std::collections::HashMap;
use std::default::Default;
use std::path::Path;

#[derive(Clone, PartialEq, Debug)]
pub struct CssSourceType {
  pub is_async: bool,
  pub ext: String,
  pub is_module: bool,
  pub file_name: String,
  pub file_path: String,
  pub source_content_map: Option<HashMap<String, String>>,
}

impl Default for CssSourceType {
  fn default() -> Self {
    CssSourceType {
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
pub fn is_css_source(filepath: &str) -> Option<CssSourceType> {
  let file = Path::new(filepath);
  let mut css_source: CssSourceType = Default::default();
  if file.is_dir() {
    return None;
  }
  if let Some(filename) = file.file_name() {
    css_source.file_path = filepath.to_string();
    css_source.ext = file
      .extension()
      .unwrap()
      .to_os_string()
      .into_string()
      .unwrap();
    return if &css_source.ext == "css"
      || &css_source.ext == "less"
      || &css_source.ext == "scss"
      || &css_source.ext == "sass"
    {
      css_source.file_name = filename.to_os_string().into_string().unwrap();
      if css_source.file_name.contains(".module.css")
        || css_source.file_name.contains(".module.less")
        || css_source.file_name.contains(".module.scss")
        || css_source.file_name.contains(".module.sass")
      {
        css_source.is_module = true
      }
      Some(css_source)
    } else {
      None
    };
  }
  None
}
