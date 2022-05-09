pub fn guess_mime_types_ext(ext: &str) -> &'static str {
  match ext {
    "jpg" => "image/jpeg",
    "jpeg" => "image/jpeg",
    "png" => "image/png",
    "gif" => "image/gif",
    "svg" => "image/svg+xml",
    "webp" => "image/web",
    _ => "unknown/unknown",
  }
}
