/*
  MIT License http://www.opensource.org/licenses/mit-license.php
  Author Natsu @xiaoxiaojx
*/

use std::{borrow::Cow, collections::HashSet, path::PathBuf, sync::Arc};

use cow_utils::CowUtils;
use futures::stream::{FuturesOrdered, StreamExt};
use once_cell::sync::Lazy;
use regex::Regex;
use rspack_fs::ReadableFileSystem;
use rspack_paths::{AssertUtf8, Utf8Path, Utf8PathBuf};
use rspack_sources::SourceMap;
use rspack_util::{base64, node_path::NodePath};

/// Source map extractor result
#[derive(Debug, Clone)]
pub struct ExtractSourceMapResult {
  pub source: String,
  pub source_map: Option<SourceMap>,
  pub file_dependencies: Option<HashSet<PathBuf>>,
}

/// Source mapping URL information
#[derive(Debug, Clone)]
pub struct SourceMappingURL {
  pub source_mapping_url: String,
  pub replacement_string: String,
}

static VALID_PROTOCOL_PATTERN: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^[a-z][a-z0-9+.-]*:").expect("Invalid regex pattern"));
static SOURCE_MAPPING_URL_REGEX: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r#"(?:/\*(?:\s*\r?\n(?://)?)?(?:\s*[#@]\s*sourceMappingURL\s*=\s*([^\s'"]*)\s*)\s*\*/|//(?:\s*[#@]\s*sourceMappingURL\s*=\s*([^\s'"]*)\s*))\s*"#).expect("Invalid regex pattern")
});
static URI_REGEX: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r"^data:([^;,]+)?((?:;[^;,]+)*?)(?:;(base64)?)?,(.*)$").expect("Invalid regex pattern")
});

/// Extract source mapping URL from code comments
pub fn get_source_mapping_url(code: &str) -> SourceMappingURL {
  // Use captures_iter to find the last match, avoiding split and collect overhead
  let mut match_result = None;
  let mut replacement_string = String::new();

  // Find the last match from the end
  if let Some(captures) = SOURCE_MAPPING_URL_REGEX.captures_iter(code).last() {
    match_result = captures
      .get(1)
      .or_else(|| captures.get(2))
      .map(|m| m.as_str());

    // Get the complete match string for replacement
    replacement_string = captures.get(0).map_or("", |m| m.as_str()).to_string();
  }

  let source_mapping_url = match_result.unwrap_or("").to_string();

  SourceMappingURL {
    source_mapping_url: if !source_mapping_url.is_empty() {
      urlencoding::decode(&source_mapping_url)
        .unwrap_or(Cow::Borrowed(&source_mapping_url))
        .to_string()
    } else {
      source_mapping_url
    },
    replacement_string,
  }
}

/// Check if value is a URL
fn is_url(value: &str) -> bool {
  VALID_PROTOCOL_PATTERN.is_match(value) && !Utf8Path::new(value).node_is_absolute_win32()
}

fn is_absolute(path: &Utf8Path) -> bool {
  path.node_is_absolute_posix() || path.node_is_absolute_win32()
}

/// Decode data URI
fn decode_data_uri(uri: &str) -> Option<String> {
  // data URL scheme: "data:text/javascript;charset=utf-8;base64,some-string"
  // http://www.ietf.org/rfc/rfc2397.txt
  let captures = URI_REGEX.captures(uri)?;
  let is_base64 = captures.get(3).is_some();
  let body = captures.get(4)?.as_str();

  if is_base64 {
    return base64::decode_to_vec(body)
      .ok()
      .and_then(|bytes| String::from_utf8(bytes).ok());
  }

  // CSS allows to use `data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg"><rect width="100%" height="100%" style="stroke: rgb(223,224,225); stroke-width: 2px; fill: none; stroke-dasharray: 6px 3px" /></svg>`
  // so we return original body if we can't `decodeURIComponent`
  match urlencoding::decode(body) {
    Ok(decoded) => Some(decoded.to_string()),
    Err(_) => Some(body.to_string()),
  }
}

/// Fetch source content from data URL
fn fetch_from_data_url(source_url: &str) -> Result<String, String> {
  if let Some(content) = decode_data_uri(source_url) {
    Ok(content)
  } else {
    Err(format!(
      "Failed to parse source map from \"data\" URL: {source_url}"
    ))
  }
}

/// Get absolute path for source file using Node.js logic
fn get_absolute_path(context: &Utf8Path, request: &str, source_root: Option<&str>) -> Utf8PathBuf {
  if let Some(source_root) = source_root {
    let source_root_path = Utf8Path::new(source_root);
    if is_absolute(source_root_path) {
      return source_root_path.join(request);
    }
    return context.join(source_root).join(request);
  }

  context.join(request)
}

/// Fetch source content from file system
async fn fetch_from_filesystem(
  fs: &Arc<dyn ReadableFileSystem>,
  source_url: &str,
) -> Result<(String, Option<String>), String> {
  if is_url(source_url) {
    return Ok((source_url.to_string(), None));
  }

  let path = PathBuf::from(source_url);
  fs.read_to_string(&path.assert_utf8())
    .await
    .map(|content| (source_url.to_string(), Some(content)))
    .map_err(|err| format!("Failed to parse source map from '{source_url}' file: {err}"))
}

/// Fetch from multiple possible file paths
async fn fetch_paths_from_filesystem(
  fs: &Arc<dyn ReadableFileSystem>,
  possible_requests: &[String],
  mut errors_accumulator: String,
) -> Result<(String, Option<String>), String> {
  if possible_requests.is_empty() {
    return Err(errors_accumulator);
  }

  // Use iteration instead of recursion to avoid Box::pin and dynamic dispatch overhead
  for (i, request) in possible_requests.iter().enumerate() {
    match fetch_from_filesystem(fs, request).await {
      Ok(result) => return Ok(result),
      Err(error) => {
        if i > 0 {
          errors_accumulator.push_str("\n\n");
        }
        errors_accumulator.push_str(&error);
      }
    }
  }

  Err(errors_accumulator)
}

/// Fetch source content from URL
async fn fetch_from_url(
  fs: &Arc<dyn ReadableFileSystem>,
  context: &Utf8Path,
  url: &str,
  source_root: Option<&str>,
  skip_reading: bool,
) -> Result<(String, Option<String>), String> {
  // 1. It's an absolute url and it is not `windows` path like `C:\dir\file`
  if is_url(url) {
    if url.starts_with("data:") {
      if skip_reading {
        return Ok((String::new(), None));
      }

      let source_content = fetch_from_data_url(url)?;
      return Ok((String::new(), Some(source_content)));
    }

    if skip_reading {
      return Ok((url.to_string(), None));
    }

    if url.starts_with("file:") {
      // Handle file:// URLs
      let path_from_url = url.strip_prefix("file://").unwrap_or(url);
      let source_url = PathBuf::from(path_from_url).to_string_lossy().into_owned();
      return fetch_from_filesystem(fs, &source_url).await;
    }

    return Err(format!(
      "Failed to parse source map: '{url}' URL is not supported"
    ));
  }

  // 2. It's a scheme-relative
  if url.starts_with("//") {
    return Err(format!(
      "Failed to parse source map: '{url}' URL is not supported"
    ));
  }

  // 3. Absolute path
  if is_absolute(Utf8Path::new(url)) {
    let source_url = url.to_string();

    if !skip_reading {
      let mut possible_requests = Vec::with_capacity(2);
      possible_requests.push(source_url.clone());

      if let Some(stripped) = url.strip_prefix('/') {
        let absolute_path = get_absolute_path(context, stripped, source_root);
        possible_requests.push(absolute_path.to_string());
      }

      return fetch_paths_from_filesystem(fs, &possible_requests, String::new()).await;
    }

    return Ok((source_url, None));
  }

  // 4. Relative path
  let source_url = get_absolute_path(context, url, source_root);
  let source_url_str = source_url.to_string();

  if !skip_reading {
    let (_, content) = fetch_from_filesystem(fs, &source_url_str).await?;
    return Ok((source_url_str, content));
  }

  Ok((source_url_str, None))
}

/// Extract source map from code content
pub async fn extract_source_map(
  fs: Arc<dyn ReadableFileSystem>,
  input: &str,
  resource_path: &str,
) -> Result<ExtractSourceMapResult, String> {
  let SourceMappingURL {
    source_mapping_url,
    replacement_string,
  } = get_source_mapping_url(input);

  if source_mapping_url.is_empty() {
    return Ok(ExtractSourceMapResult {
      source: input.to_string(),
      source_map: None,
      file_dependencies: None,
    });
  }

  let base_context = Utf8Path::new(resource_path)
    .parent()
    .ok_or_else(|| "Invalid resource path".to_string())?;

  let (source_url, source_content) =
    fetch_from_url(&fs, base_context, &source_mapping_url, None, false).await?;

  let content = match source_content.as_deref() {
    Some(c) => c.trim_start_matches(")]}'"),
    None => {
      return Ok(ExtractSourceMapResult {
        source: input.to_string(),
        source_map: None,
        file_dependencies: if source_url.is_empty() {
          None
        } else {
          let mut set = HashSet::new();
          set.insert(PathBuf::from(source_url));
          Some(set)
        },
      });
    }
  };

  // Create SourceMap directly from JSON
  let mut source_map =
    SourceMap::from_json(content).map_err(|e| format!("Failed to parse source map: {e}"))?;

  let context = if !source_url.is_empty() {
    Utf8Path::new(&source_url).parent().unwrap_or(base_context)
  } else {
    base_context
  };

  let mut resolved_sources = Vec::new();
  let mut file_dependencies = if source_url.is_empty() {
    None
  } else {
    let mut set = HashSet::new();
    set.insert(PathBuf::from(&source_url));
    Some(set)
  };

  // Get sources from SourceMap and take ownership
  let sources = source_map.sources().to_vec();
  let source_root = source_map.source_root().map(|s| s.to_string());

  // Pre-collect all source content to avoid borrowing issues
  let source_contents: Vec<Option<String>> = (0..sources.len())
    .map(|i| source_map.get_source_content(i).map(|s| s.to_string()))
    .collect();

  // Process sources in parallel using FuturesOrdered to maintain order
  let mut futures = FuturesOrdered::new();

  // Use zip to consume both vectors without extra cloning
  for (source, original_content) in sources.into_iter().zip(source_contents.into_iter()) {
    let skip_reading = original_content.is_some();
    let source_root = source_root.clone();
    let context = context.to_path_buf();

    let fs = fs.clone();
    futures.push_back(async move {
      let result =
        fetch_from_url(&fs, &context, &source, source_root.as_deref(), skip_reading).await;
      (original_content, skip_reading, result)
    });
  }

  // Collect results in order
  while let Some((original_content, skip_reading, result)) = futures.next().await {
    let (source_url_result, source_content_result) = result?;

    let final_content = if skip_reading {
      original_content
    } else {
      source_content_result
    };

    if !skip_reading && !source_url_result.is_empty() && !is_url(&source_url_result) {
      if let Some(ref mut deps) = file_dependencies {
        deps.insert(PathBuf::from(&source_url_result));
      } else {
        let mut set = HashSet::new();
        set.insert(PathBuf::from(&source_url_result));
        file_dependencies = Some(set);
      }
    }

    resolved_sources.push((source_url_result, final_content));
  }

  // Build the final SourceMap using setter methods - consume resolved_sources to avoid cloning
  let (sources_vec, sources_content_vec): (Vec<String>, Vec<String>) = resolved_sources
    .into_iter()
    .map(|(url, content)| (url, content.unwrap_or_default()))
    .unzip();

  source_map.set_sources(sources_vec);
  source_map.set_sources_content(sources_content_vec);

  // Remove source_root as per original logic
  source_map.set_source_root(None::<String>);

  // Optimize string replacement to avoid unnecessary cloning
  let new_source = if replacement_string.is_empty() {
    input.to_string()
  } else {
    input.cow_replace(&replacement_string, "").into_owned()
  };

  Ok(ExtractSourceMapResult {
    source: new_source,
    source_map: Some(source_map),
    file_dependencies,
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_get_source_mapping_url() {
    // Test cases based on expected results from extractSourceMap.unittest.js.snap
    let test_cases = vec![
      (
        "/*#sourceMappingURL=absolute-sourceRoot-source-map.map*/",
        "absolute-sourceRoot-source-map.map",
      ),
      (
        "/*  #sourceMappingURL=absolute-sourceRoot-source-map.map  */",
        "absolute-sourceRoot-source-map.map",
      ),
      (
        "//#sourceMappingURL=absolute-sourceRoot-source-map.map",
        "absolute-sourceRoot-source-map.map",
      ),
      (
        "//@sourceMappingURL=absolute-sourceRoot-source-map.map",
        "absolute-sourceRoot-source-map.map",
      ),
      (
        " //  #sourceMappingURL=absolute-sourceRoot-source-map.map",
        "absolute-sourceRoot-source-map.map",
      ),
      (
        " //  #  sourceMappingURL  =   absolute-sourceRoot-source-map.map  ",
        "absolute-sourceRoot-source-map.map",
      ),
      (
        "// #sourceMappingURL = http://hello.com/external-source-map2.map",
        "http://hello.com/external-source-map2.map",
      ),
      (
        "// #sourceMappingURL = //hello.com/external-source-map2.map",
        "//hello.com/external-source-map2.map",
      ),
      (
        "// @sourceMappingURL=data:application/source-map;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiaW5saW5lLXNvdXJjZS1tYXAuanMiLCJzb3VyY2VzIjpbImlubGluZS1zb3VyY2UtbWFwLnR4dCJdLCJzb3VyY2VzQ29udGVudCI6WyJ3aXRoIFNvdXJjZU1hcCJdLCJtYXBwaW5ncyI6IkFBQUEifQ==",
        "data:application/source-map;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiaW5saW5lLXNvdXJjZS1tYXAuanMiLCJzb3VyY2VzIjpbImlubGluZS1zb3VyY2UtbWFwLnR4dCJdLCJzb3VyY2VzQ29udGVudCI6WyJ3aXRoIFNvdXJjZU1hcCJdLCJtYXBwaW5ncyI6IkFBQUEifQ==",
      ),
      (
        r#"
        with SourceMap

        // #sourceMappingURL = /sample-source-map.map
        // comment
        "#,
        "/sample-source-map.map",
      ),
      (
        r#"
        with SourceMap
        // #sourceMappingURL = /sample-source-map-1.map
        // #sourceMappingURL = /sample-source-map-2.map
        // #sourceMappingURL = /sample-source-map-last.map
        // comment
        "#,
        "/sample-source-map-last.map",
      ),
      // JavaScript code snippet with variable reference, expected to return empty string
      (
        r#""
        /*# sourceMappingURL=data:application/json;base64,"+btoa(unescape(encodeURIComponent(JSON.stringify(sourceMap))))+" */";"#,
        "",
      ),
      // JavaScript code snippet, expected to truncate at first variable reference
      (
        r#"// # sourceMappingURL=data:application/json;base64,"+btoa(unescape(encodeURIComponent(JSON.stringify(sourceMap))))+"'"#,
        "data:application/json;base64,",
      ),
      // JavaScript code snippet with variable reference, expected to return empty string
      (
        r#"anInvalidDirective = "
/*# sourceMappingURL=data:application/json;base64,"+btoa(unescape(encodeURIComponent(JSON.stringify(sourceMap))))+" */";"#,
        "",
      ),
    ];

    for (input, expected) in test_cases {
      let result = get_source_mapping_url(input);
      assert_eq!(
        result.source_mapping_url, expected,
        "Failed for input: {input}"
      );
    }
  }

  #[test]
  fn test_get_source_mapping_url_empty_cases() {
    // Test cases without sourceMappingURL
    let cases = vec![
      "const foo = 'bar';",
      "// This is a regular comment",
      "/* Multi-line\n   comment\n   without sourceMappingURL */",
      "",
    ];

    for case in cases {
      let result = get_source_mapping_url(case);
      assert!(result.source_mapping_url.is_empty());
      assert!(result.replacement_string.is_empty());
    }
  }
}
