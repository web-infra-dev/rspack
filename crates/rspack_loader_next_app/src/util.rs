use std::borrow::Cow;

use cow_utils::CowUtils;

pub fn normalize_app_path(route: &str) -> String {
  let segments = route.split('/');
  let segments_len = segments.clone().count();
  let mut pathname = String::new();

  for (index, segment) in segments.enumerate() {
    // Empty segments are ignored.
    if segment.is_empty() {
      continue;
    }

    // Groups are ignored.
    if is_group_segment(segment) {
      continue;
    }

    // Parallel segments are ignored.
    if segment.starts_with('@') {
      continue;
    }

    // The last segment (if it's a leaf) should be ignored.
    if (segment == "page" || segment == "route") && index == segments_len - 1 {
      continue;
    }

    pathname.push('/');
    pathname.push_str(segment);
  }

  ensure_leading_slash(&pathname)
}

fn ensure_leading_slash(path: &str) -> String {
  if path.starts_with('/') {
    path.to_string()
  } else {
    format!("/{}", path)
  }
}

pub fn is_group_segment(segment: &str) -> bool {
  segment.starts_with('(') && segment.ends_with(')')
}

pub fn normalize_underscore(pathname: &str) -> String {
  pathname.replace("%5F", "_")
}
