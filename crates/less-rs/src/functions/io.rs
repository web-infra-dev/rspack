//! Functions that need the file system or produce data URIs:
//! `data-uri`, `image-size`, `image-width`, `image-height`, `svg-gradient`.

use std::path::{Path, PathBuf};

use cow_utils::CowUtils;

use super::{
  Registry,
  builtins::{encode_uri_component, undefined_value},
};
use crate::{
  color,
  css::{self, CssCtx},
  error::{LResult, LessError},
  eval::EvalCtx,
  tree::*,
};

type R<'a> = LResult<Option<Node<'a>>>;

pub(super) fn register(r: &mut Registry) {
  r.add("svg-gradient", svg_gradient);
  r.add("data-uri", data_uri);
  r.add("image-size", |c, p, a| {
    let (w, h) = image_size(c, p, a)?;
    let wd = c.a.dimension_str(w, "px");
    let hd = c.a.dimension_str(h, "px");
    Ok(Some(Node::Expression(c.a.expression(&[wd, hd], None))))
  });
  r.add("image-width", |c, p, a| {
    let (w, _) = image_size(c, p, a)?;
    Ok(Some(c.a.dimension_str(w, "px")))
  });
  r.add("image-height", |c, p, a| {
    let (_, h) = image_size(c, p, a)?;
    Ok(Some(c.a.dimension_str(h, "px")))
  });
}

fn url_node<'a>(c: &mut EvalCtx<'a, '_>, pos: Pos, quote: &'a str, value: &str) -> Node<'a> {
  let q = c.a.quoted(quote, c.str(value), false, pos);
  Node::Url(c.a.alloc(Url {
    value: Node::Quoted(q),
    pos,
    is_evald: false,
  }))
}

fn svg_gradient<'a>(c: &mut EvalCtx<'a, '_>, pos: Pos, a: &[Node<'a>]) -> R<'a> {
  let throw_desc = || {
    LessError::new(
      "Argument",
      "svg-gradient expects direction, start_color [start_position], [color position,]..., end_color [end_position] or direction, color list",
    )
  };
  let direction = a.first().copied().ok_or_else(|| undefined_value("toCSS"))?;
  let mut render = CssCtx::empty();
  let direction_value = css::to_css(direction, &mut render)?;
  let stops: Vec<Node<'a>> = if a.len() == 2 {
    match a[1].array_value() {
      Some(v) if v.len() >= 2 => v.to_vec(),
      Some(_) => return Err(throw_desc()),
      // a string `.value` (e.g. a color keyword) passes the length
      // check; its characters then fail the color check.
      None => match a[1].str_value() {
        Some(v) if v.encode_utf16().count() >= 2 => vec![c.a.keyword("x"); 2],
        _ => return Err(throw_desc()),
      },
    }
  } else if a.len() < 3 {
    return Err(throw_desc());
  } else {
    a[1..].to_vec()
  };
  let mut gradient_type = "linear";
  let mut rect = "x=\"0\" y=\"0\" width=\"1\" height=\"1\"";
  let dir = match direction_value.as_str() {
    "to bottom" => "x1=\"0%\" y1=\"0%\" x2=\"0%\" y2=\"100%\"",
    "to right" => "x1=\"0%\" y1=\"0%\" x2=\"100%\" y2=\"0%\"",
    "to bottom right" => "x1=\"0%\" y1=\"0%\" x2=\"100%\" y2=\"100%\"",
    "to top right" => "x1=\"0%\" y1=\"100%\" x2=\"100%\" y2=\"0%\"",
    "ellipse" | "ellipse at center" => {
      gradient_type = "radial";
      rect = "x=\"-50\" y=\"-50\" width=\"101\" height=\"101\"";
      "cx=\"50%\" cy=\"50%\" r=\"75%\""
    }
    _ => {
      return Err(LessError::new(
        "Argument",
        "svg-gradient direction must be 'to bottom', 'to right', 'to bottom right', 'to top right' or 'ellipse at center'",
      ));
    }
  };
  let mut out = format!(
    "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 1 1\"><{}Gradient id=\"g\" {}>",
    gradient_type, dir
  );
  let n = stops.len();
  for (i, stop) in stops.iter().enumerate() {
    let (color, position) = match stop {
      Node::Expression(e) => (e.value.first().copied(), e.value.get(1).copied()),
      other => (Some(*other), None),
    };
    let color = match color {
      Some(Node::Color(col)) => col,
      _ => return Err(throw_desc()),
    };
    let pos_ok = ((i == 0 || i + 1 == n) && position.is_none())
      || matches!(position, Some(Node::Dimension(_)));
    if !pos_ok {
      return Err(throw_desc());
    }
    let position_value = match position {
      Some(p) => css::to_css(p, &mut render)?,
      None => (if i == 0 { "0%" } else { "100%" }).to_string(),
    };
    let alpha = color.alpha;
    out.push_str(&format!(
      "<stop offset=\"{}\" stop-color=\"{}\"{}/>",
      position_value,
      color::to_hex(&color.rgb),
      if alpha < 1.0 {
        format!(" stop-opacity=\"{}\"", crate::num::to_js_string(alpha))
      } else {
        String::new()
      }
    ));
  }
  out.push_str(&format!(
    "</{}Gradient><rect {} fill=\"url(#g)\" /></svg>",
    gradient_type, rect
  ));
  let encoded = format!("data:image/svg+xml,{}", encode_uri_component(&out));
  Ok(Some(url_node(c, pos, "'", &encoded)))
}

/// Resolves a file path for `data-uri`/`image-size` the way the node file
/// manager does: relative to the current directory (or entry path), then the
/// include paths.
fn resolve_file(c: &EvalCtx<'_, '_>, pos: Pos, file_path: &str) -> Option<PathBuf> {
  let info = c.s.pos_file_info(pos);
  let current_directory = match &info {
    Some(i) => {
      if i.rewrite_urls != crate::options::RewriteUrls::Off {
        i.current_directory
      } else {
        i.entry_path
      }
    }
    None => "",
  };
  let p = Path::new(file_path);
  let mut candidates: Vec<PathBuf> = Vec::new();
  if p.is_absolute() {
    candidates.push(p.to_path_buf());
  } else {
    candidates.push(Path::new(current_directory).join(p));
    for inc in &c.s.opts.include_paths {
      candidates.push(Path::new(inc).join(p));
    }
    candidates.push(p.to_path_buf());
  }
  candidates.into_iter().find(|cand| c.s.fs.exists(cand))
}

fn mime_lookup(path: &str) -> &'static str {
  let ext = path
    .rsplit('.')
    .next()
    .unwrap_or("")
    .cow_to_ascii_lowercase();
  match ext.as_ref() {
    "png" => "image/png",
    "jpg" | "jpeg" | "jpe" => "image/jpeg",
    "gif" => "image/gif",
    "svg" | "svgz" => "image/svg+xml",
    "webp" => "image/webp",
    "bmp" => "image/bmp",
    "ico" => "image/x-icon",
    "tif" | "tiff" => "image/tiff",
    "woff" => "font/woff",
    "woff2" => "font/woff2",
    "ttf" => "font/ttf",
    "otf" => "font/otf",
    "eot" => "application/vnd.ms-fontobject",
    "css" => "text/css",
    "html" | "htm" => "text/html",
    "txt" | "text" => "text/plain",
    "js" | "mjs" => "application/javascript",
    "json" => "application/json",
    "xml" => "application/xml",
    "pdf" => "application/pdf",
    _ => "application/octet-stream",
  }
}

fn charset_lookup(mime: &str) -> Option<&'static str> {
  if mime.starts_with("text/") || mime == "application/javascript" || mime == "application/json" {
    Some("UTF-8")
  } else {
    None
  }
}

fn data_uri<'a>(c: &mut EvalCtx<'a, '_>, pos: Pos, a: &[Node<'a>]) -> R<'a> {
  let (mut mimetype_node, mut file_node) = (a.first().copied(), a.get(1).copied());
  if file_node.is_none() {
    file_node = mimetype_node;
    mimetype_node = None;
  }
  let file_node = file_node.ok_or_else(|| undefined_value("value"))?;
  let mut file_path = crate::eval::js_string_value(file_node)
    .unwrap_or("")
    .to_string();
  let mut fragment = String::new();
  if let Some(i) = file_path.find('#') {
    fragment = file_path[i..].to_string();
    file_path.truncate(i);
  }
  let fallback = |c: &mut EvalCtx<'a, '_>, n: Node<'a>| -> R<'a> {
    let u = c.a.alloc(Url {
      value: n,
      pos,
      is_evald: false,
    });
    Ok(Some(c.eval(Node::Url(u))?))
  };
  let use_base64;
  let mut mimetype = mimetype_node
    .and_then(crate::eval::js_string_value)
    .unwrap_or("")
    .to_string();
  if mimetype_node.is_none() {
    mimetype = mime_lookup(&file_path).to_string();
    if mimetype == "image/svg+xml" {
      use_base64 = false;
    } else {
      let cs = charset_lookup(&mimetype);
      use_base64 = !matches!(cs, Some("US-ASCII") | Some("UTF-8"));
    }
    if use_base64 {
      mimetype.push_str(";base64");
    }
  } else {
    use_base64 = mimetype.ends_with(";base64");
  }
  let resolved = resolve_file(c, pos, &file_path);
  let bytes = match resolved.and_then(|p| c.s.fs.read_bytes(&p).ok()) {
    Some(b) => b,
    None => {
      c.s.warn(
        format!(
          "Skipped data-uri embedding of {} because file not found",
          file_path
        ),
        None,
      );
      return fallback(c, file_node);
    }
  };
  let buf = if use_base64 {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(&bytes)
  } else {
    encode_uri_component(&String::from_utf8_lossy(&bytes))
  };
  let uri = format!("data:{},{}{}", mimetype, buf, fragment);
  Ok(Some(url_node(c, pos, "\"", &uri)))
}

fn image_size<'a>(c: &mut EvalCtx<'a, '_>, pos: Pos, a: &[Node<'a>]) -> LResult<(f64, f64)> {
  let node = a.first().copied().ok_or_else(|| undefined_value("value"))?;
  let mut file_path = crate::eval::js_string_value(node).unwrap_or("").to_string();
  if let Some(i) = file_path.find('#') {
    file_path.truncate(i);
  }
  let resolved = resolve_file(c, pos, &file_path)
    .ok_or_else(|| LessError::new("File", format!("'{}' wasn't found", file_path)))?;
  let bytes = c
    .s
    .fs
    .read_bytes(&resolved)
    .map_err(|e| LessError::new("File", e.to_string()))?;
  match probe_image_size(&bytes) {
    Some((w, h)) => Ok((w as f64, h as f64)),
    None => Err(LessError::new(
      "File",
      format!("Unrecognised image format for '{}'", file_path),
    )),
  }
}

/// Minimal image header probing (PNG, GIF, JPEG, WebP, SVG).
pub(crate) fn probe_image_size(b: &[u8]) -> Option<(u32, u32)> {
  // PNG
  if b.len() >= 24 && &b[..8] == b"\x89PNG\r\n\x1a\n" {
    let w = u32::from_be_bytes([b[16], b[17], b[18], b[19]]);
    let h = u32::from_be_bytes([b[20], b[21], b[22], b[23]]);
    return Some((w, h));
  }
  // GIF
  if b.len() >= 10 && (&b[..6] == b"GIF87a" || &b[..6] == b"GIF89a") {
    let w = u16::from_le_bytes([b[6], b[7]]) as u32;
    let h = u16::from_le_bytes([b[8], b[9]]) as u32;
    return Some((w, h));
  }
  // JPEG
  if b.len() >= 4 && b[0] == 0xFF && b[1] == 0xD8 {
    let mut i = 2;
    while i + 9 < b.len() {
      if b[i] != 0xFF {
        i += 1;
        continue;
      }
      let marker = b[i + 1];
      if marker == 0xD8 || marker == 0x01 || (0xD0..=0xD7).contains(&marker) {
        i += 2;
        continue;
      }
      let len = u16::from_be_bytes([b[i + 2], b[i + 3]]) as usize;
      if matches!(marker, 0xC0..=0xC3 | 0xC5..=0xC7 | 0xC9..=0xCB | 0xCD..=0xCF) {
        let h = u16::from_be_bytes([b[i + 5], b[i + 6]]) as u32;
        let w = u16::from_be_bytes([b[i + 7], b[i + 8]]) as u32;
        return Some((w, h));
      }
      i += 2 + len;
    }
    return None;
  }
  // WebP
  if b.len() >= 30 && &b[..4] == b"RIFF" && &b[8..12] == b"WEBP" {
    match &b[12..16] {
      b"VP8 " => {
        let w = (u16::from_le_bytes([b[26], b[27]]) & 0x3fff) as u32;
        let h = (u16::from_le_bytes([b[28], b[29]]) & 0x3fff) as u32;
        return Some((w, h));
      }
      b"VP8L" => {
        let bits = u32::from_le_bytes([b[21], b[22], b[23], b[24]]);
        return Some(((bits & 0x3fff) + 1, ((bits >> 14) & 0x3fff) + 1));
      }
      b"VP8X" => {
        let w = 1 + (b[24] as u32 | (b[25] as u32) << 8 | (b[26] as u32) << 16);
        let h = 1 + (b[27] as u32 | (b[28] as u32) << 8 | (b[29] as u32) << 16);
        return Some((w, h));
      }
      _ => return None,
    }
  }
  // SVG: width/height attributes on the root element.
  let text = String::from_utf8_lossy(&b[..b.len().min(4096)]);
  if let Some(start) = text.find("<svg") {
    let tag_end = text[start..]
      .find('>')
      .map(|e| start + e)
      .unwrap_or(text.len());
    let tag = &text[start..tag_end];
    let attr = |name: &str| -> Option<f64> {
      let key = format!(" {}=", name);
      let p = tag.find(&key)? + key.len();
      let q = tag.as_bytes().get(p).copied()?;
      let rest = &tag[p + 1..];
      let end = rest.find(q as char)?;
      let v = &rest[..end];
      let num: String = v
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
      num.parse().ok()
    };
    if let (Some(w), Some(h)) = (attr("width"), attr("height")) {
      return Some((w as u32, h as u32));
    }
    if let Some(p) = tag.find("viewBox=") {
      let q = tag.as_bytes()[p + 8] as char;
      let rest = &tag[p + 9..];
      if let Some(end) = rest.find(q) {
        let nums: Vec<f64> = rest[..end]
          .split([' ', ','])
          .filter_map(|s| s.parse().ok())
          .collect();
        if nums.len() == 4 {
          return Some((nums[2] as u32, nums[3] as u32));
        }
      }
    }
  }
  None
}
