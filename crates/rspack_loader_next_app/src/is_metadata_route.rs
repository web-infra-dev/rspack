#[allow(unused_variables)]
use std::collections::HashMap;

use lazy_regex::Lazy;
use regex::Regex;
use rspack_util::fx_hash::BuildFxHasher;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PossibleImageFileNameConvention {
  Icon,
  Apple,
  Favicon,
  Twitter,
  OpenGraph,
  Manifest,
}

impl PossibleImageFileNameConvention {
  pub fn as_str(&self) -> &'static str {
    match self {
      PossibleImageFileNameConvention::Icon => "icon",
      PossibleImageFileNameConvention::Apple => "apple",
      PossibleImageFileNameConvention::Favicon => "favicon",
      PossibleImageFileNameConvention::Twitter => "twitter",
      PossibleImageFileNameConvention::OpenGraph => "opengraph",
      PossibleImageFileNameConvention::Manifest => "manifest",
    }
  }
}

pub struct MetadataImage {
  pub filename: &'static str,
  pub extensions: &'static [&'static str],
}

pub type StaticMetadataImages =
  HashMap<PossibleImageFileNameConvention, MetadataImage, BuildFxHasher>;

pub static STATIC_METADATA_IMAGES: Lazy<StaticMetadataImages> = Lazy::new(|| {
  let mut map: StaticMetadataImages = Default::default();
  map.insert(
    PossibleImageFileNameConvention::Icon,
    MetadataImage {
      filename: "icon",
      extensions: &["ico", "jpg", "jpeg", "png", "svg"],
    },
  );
  map.insert(
    PossibleImageFileNameConvention::Apple,
    MetadataImage {
      filename: "apple-icon",
      extensions: &["jpg", "jpeg", "png"],
    },
  );
  map.insert(
    PossibleImageFileNameConvention::Favicon,
    MetadataImage {
      filename: "favicon",
      extensions: &["ico"],
    },
  );
  map.insert(
    PossibleImageFileNameConvention::OpenGraph,
    MetadataImage {
      filename: "opengraph-image",
      extensions: &["jpg", "jpeg", "png", "gif"],
    },
  );
  map.insert(
    PossibleImageFileNameConvention::Twitter,
    MetadataImage {
      filename: "twitter-image",
      extensions: &["jpg", "jpeg", "png", "gif"],
    },
  );
  map
});

pub fn get_extension_regex_string(
  static_extensions: &[&str],
  dynamic_extensions: Option<&[&str]>,
) -> String {
  if dynamic_extensions.is_none() {
    return format!("\\.(?:{})", static_extensions.join("|"));
  }
  let dynamic_extensions = dynamic_extensions.unwrap();
  format!(
    "(?:\\.({})|((\\[\\])?\\.({})))",
    static_extensions.join("|"),
    dynamic_extensions.join("|")
  )
}

pub fn is_metadata_route_file(
  app_dir_relative_path: &str,
  page_extensions: &[&str],
  with_extension: bool,
) -> bool {
  let metadata_route_files_regex = vec![
    {
      let mut page_extensions = page_extensions.to_vec();
      page_extensions.push("txt");
      Regex::new(&format!(
        r"^[\\/]robots{}",
        if with_extension {
          get_extension_regex_string(&page_extensions, None)
        } else {
          String::new()
        }
      ))
      .unwrap()
    },
    {
      let mut page_extensions = page_extensions.to_vec();
      page_extensions.push("webmanifest");
      page_extensions.push("json");
      Regex::new(&format!(
        r"^[\\/]manifest{}",
        if with_extension {
          get_extension_regex_string(&page_extensions, None)
        } else {
          String::new()
        }
      ))
      .unwrap()
    },
    Regex::new(r"^[\\/]{favicon}\.ico$").unwrap(),
    Regex::new(&format!(
      r"[\\/]sitemap{}",
      if with_extension {
        get_extension_regex_string(&["xml"], Some(page_extensions))
      } else {
        String::new()
      }
    ))
    .unwrap(),
    {
      let metadata = STATIC_METADATA_IMAGES
        .get(&PossibleImageFileNameConvention::Icon)
        .unwrap();
      Regex::new(&format!(
        r"[\\/]{}\\d?{}",
        metadata.filename,
        if with_extension {
          get_extension_regex_string(metadata.extensions, Some(page_extensions))
        } else {
          String::new()
        }
      ))
      .unwrap()
    },
    {
      let metadata = STATIC_METADATA_IMAGES
        .get(&PossibleImageFileNameConvention::Apple)
        .unwrap();
      Regex::new(&format!(
        r"[\\/]{}\\d?{}",
        metadata.filename,
        if with_extension {
          get_extension_regex_string(metadata.extensions, Some(page_extensions))
        } else {
          String::new()
        }
      ))
      .unwrap()
    },
    {
      let metadata = STATIC_METADATA_IMAGES
        .get(&PossibleImageFileNameConvention::OpenGraph)
        .unwrap();
      Regex::new(&format!(
        r"[\\/]{}\\d?{}",
        metadata.filename,
        if with_extension {
          get_extension_regex_string(metadata.extensions, Some(page_extensions))
        } else {
          String::new()
        }
      ))
      .unwrap()
    },
    {
      let metadata = STATIC_METADATA_IMAGES
        .get(&PossibleImageFileNameConvention::Twitter)
        .unwrap();
      Regex::new(&format!(
        r"[\\/]{}\\d?{}",
        metadata.filename,
        if with_extension {
          get_extension_regex_string(metadata.extensions, Some(page_extensions))
        } else {
          String::new()
        }
      ))
      .unwrap()
    },
  ];

  let normalized_app_dir_relative_path = normalize_path_sep(app_dir_relative_path);
  metadata_route_files_regex
    .iter()
    .any(|r| r.is_match(&normalized_app_dir_relative_path))
}

pub fn is_static_metadata_route_file(app_dir_relative_path: &str) -> bool {
  is_metadata_route_file(app_dir_relative_path, &vec![], true)
}

pub fn is_static_metadata_route(page: &str) -> bool {
  page == "/robots" || page == "/manifest" || is_static_metadata_route_file(page)
}

pub fn is_metadata_route(route: &str) -> bool {
  let mut page = route.replace("^/?app/", "").replace("/route$", "");
  if !page.starts_with('/') {
    page = format!("/{}", page);
  }

  !page.ends_with("/page") && is_metadata_route_file(&page, &DEFAULT_EXTENSIONS, false)
}

pub static DEFAULT_EXTENSIONS: Lazy<Vec<&'static str>> =
  Lazy::new(|| vec!["js", "jsx", "ts", "tsx"]);

fn normalize_path_sep(path: &str) -> String {
  path.replace("\\", "/")
}

#[cfg(test)]
mod tests {
  use super::*;

  fn create_extension_match_regex(
    static_extensions: &[&str],
    dynamic_extensions: Option<&[&str]>,
  ) -> Regex {
    Regex::new(&format!(
      "^{}$",
      get_extension_regex_string(static_extensions, dynamic_extensions)
    ))
    .unwrap()
  }

  #[test]
  fn test_with_dynamic_extensions() {
    let regex = create_extension_match_regex(&["png", "jpg"], Some(&["tsx", "ts"]));
    assert!(regex.is_match(".png"));
    assert!(regex.is_match(".jpg"));
    assert!(!regex.is_match(".webp"));

    assert!(regex.is_match(".tsx"));
    assert!(regex.is_match(".ts"));
    assert!(!regex.is_match(".js"));
  }

  #[test]
  fn test_match_dynamic_multi_routes_with_dynamic_extensions() {
    let regex = create_extension_match_regex(&["png"], Some(&["ts"]));
    assert!(regex.is_match(".png"));
    assert!(!regex.is_match("[].png"));

    assert!(regex.is_match(".ts"));
    assert!(regex.is_match("[].ts"));
    assert!(!regex.is_match(".tsx"));
    assert!(!regex.is_match("[].tsx"));
  }

  #[test]
  fn test_without_dynamic_extensions() {
    let regex = create_extension_match_regex(&["png", "jpg"], None);
    assert!(regex.is_match(".png"));
    assert!(regex.is_match(".jpg"));
    assert!(!regex.is_match(".webp"));

    assert!(!regex.is_match(".tsx"));
    assert!(!regex.is_match(".js"));
  }
}
