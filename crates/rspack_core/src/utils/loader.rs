use std::path::Path;

use crate::{Loader, ResolvedLoadedFile};

pub fn interpret_loaded_file_to_js(file: ResolvedLoadedFile, id: &str) -> ResolvedLoadedFile {
  let loader = file.loader;

  match loader {
    Loader::DataURI => {
      let ext = Path::new(id)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("unknown");
      let mime_type = guess_mime_types_ext(ext);
      let format = "base64";
      let data_uri = format!(
        "data:{};{},{}",
        mime_type,
        format,
        base64::encode(&file.content)
      );
      ResolvedLoadedFile::with_loader(
        format!(
          "
        var img = \"{}\";
        export default img;
        ",
          data_uri
        )
        .trim()
        .to_string(),
        Loader::Js,
      )
    }
    Loader::Json => {
      let data = file.content;
      ResolvedLoadedFile::with_loader(
        format!(
          "
        export default {}
        ",
          data
        ),
        Loader::Js,
      )
    }
    Loader::Text => {
      let data = file.content;
      let data = serde_json::to_string(&data)
        .ok()
        .unwrap_or_else(|| panic!("jsonify content of file {:?} failed", id));
      ResolvedLoadedFile::with_loader(
        format!(
          "
        export default {}
        ",
          data
        ),
        Loader::Js,
      )
    }
    Loader::Null => ResolvedLoadedFile::with_loader(
      r#"
    export default {}
    "#
      .to_string(),
      Loader::Js,
    ),
    _ => file,
  }
}

pub fn guess_mime_types_ext(ext: &str) -> &'static str {
  match ext {
    "jpg" => "image/jpeg",
    "jpeg" => "image/jpeg",
    "png" => "image/png",
    "gif" => "image/gif",
    "svg" => "image/svg+xml",
    "webp" => "image/webp",
    _ => "unknown/unknown",
  }
}
