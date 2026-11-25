use std::{borrow::Cow, collections::BTreeMap};

use cow_utils::CowUtils;
use rspack_error::{Result, ToStringResultToRspackResultExt, error};
use serde_json::{Map, Value, json};
use swc_core::common::DUMMY_SP;
use swc_html::ast::{Child, Element, Namespace, Text};
use url::{ParseError, Url};

use crate::{
  parser::CompiledDocument,
  tag::{HtmlPluginAttribute, HtmlPluginTag},
};

pub(crate) fn add_integrity_to_import_map(
  document: &mut CompiledDocument,
  integrity: BTreeMap<String, String>,
) -> Result<()> {
  let children = match document {
    CompiledDocument::Document(document) | CompiledDocument::DocumentWithoutDoctype(document) => {
      &mut document.children
    }
    CompiledDocument::DocumentFragment(fragment) => &mut fragment.children,
  };
  let (nonce, late_base) = find_import_map_context(children);
  let nonce = nonce.map(str::to_owned);
  let base_integrity = late_base.map_or_else(BTreeMap::new, |base| {
    rebase_integrity_urls(&integrity, base)
  });

  let mut seen_module_fetch = false;
  if merge_existing_map(
    children,
    &integrity,
    &base_integrity,
    &mut seen_module_fetch,
  )? || !seen_module_fetch
  {
    return Ok(());
  }

  let contents = serialize_map(&json!({ "integrity": integrity }))?;
  insert_before_module_fetch(children, &contents, nonce.as_deref());
  Ok(())
}

fn attribute<'a>(element: &'a Element, name: &str) -> Option<&'a str> {
  element
    .attributes
    .iter()
    .find(|attribute| attribute.name == name)
    .and_then(|attribute| attribute.value.as_deref())
}

fn is_script(element: &Element, script_type: &str) -> bool {
  element.tag_name == "script"
    && attribute(element, "type").is_some_and(|value| value.eq_ignore_ascii_case(script_type))
}

fn starts_module_fetch(element: &Element) -> bool {
  is_script(element, "module")
    || (element.tag_name == "link"
      && attribute(element, "rel").is_some_and(|value| {
        value
          .split_ascii_whitespace()
          .any(|rel| rel.eq_ignore_ascii_case("modulepreload"))
      }))
}

fn is_active_html_element(element: &Element) -> bool {
  element.namespace == Namespace::HTML && !matches!(&*element.tag_name, "template" | "noscript")
}

fn find_import_map_context(children: &[Child]) -> (Option<&str>, Option<&str>) {
  let mut pending = vec![children.iter()];
  let mut preload_nonce = None;
  let mut seen_import_map = false;
  let mut seen_module_fetch = false;
  let mut base_href = None;
  let mut late_base = None;

  while let Some(children) = pending.last_mut() {
    let Some(child) = children.next() else {
      pending.pop();
      continue;
    };
    let Child::Element(element) = child else {
      continue;
    };
    if !is_active_html_element(element) {
      continue;
    }
    pending.push(element.children.iter());

    if starts_module_fetch(element) {
      let first_module_fetch = !seen_module_fetch;
      seen_module_fetch = true;
      if let Some(nonce) = attribute(element, "nonce").filter(|nonce| !nonce.is_empty()) {
        if first_module_fetch || is_script(element, "module") {
          return (Some(nonce), late_base);
        }
        preload_nonce.get_or_insert(nonce);
      }
    } else if !seen_module_fetch {
      if base_href.is_none() && element.tag_name == "base" {
        // The first base with an href wins, including an empty href.
        base_href = element
          .attributes
          .iter()
          .find(|attribute| attribute.name == "href")
          .map(|attribute| attribute.value.as_deref().unwrap_or_default());
        if seen_import_map {
          late_base = base_href;
        }
      }
      seen_import_map |= is_script(element, "importmap");
    }
  }

  (preload_nonce, late_base)
}

// Preserve the authored map's base URL. Only its generated integrity keys
// need to refer to the base URL in effect when the modules are fetched.
fn rebase_integrity_urls(
  integrity: &BTreeMap<String, String>,
  base: &str,
) -> BTreeMap<String, String> {
  let base = base
    .trim_matches(|c: char| c <= ' ')
    .cow_replace(&['\t', '\r', '\n'][..], "");
  match Url::parse(&base) {
    Ok(base) if !base.cannot_be_a_base() => {
      return integrity_aliases(integrity, |url| base.join(url).ok().map(String::from));
    }
    Err(ParseError::RelativeUrlWithoutBase) => {}
    _ => return BTreeMap::new(),
  }

  // Keep relative bases independent of the document's deployment URL.
  let path = base
    .split(['?', '#'])
    .next()
    .unwrap_or_default()
    .cow_replace('\\', "/");
  let (authority, path) = if path.starts_with("//") {
    let authority_and_path = path.trim_start_matches('/');
    let (authority, path) = authority_and_path
      .split_once('/')
      .unwrap_or((authority_and_path, ""));
    if Url::parse(&format!("https://{authority}")).is_err() {
      return BTreeMap::new();
    }
    // Keep explicit ports because the document's scheme is unknown.
    (format!("//{authority}"), Cow::Owned(format!("/{path}")))
  } else {
    (String::new(), path)
  };
  let filename = path.rsplit('/').next().unwrap_or_default();
  let directory = if matches!(
    filename.cow_to_ascii_lowercase().as_ref(),
    "." | ".." | "%2e" | "%2e%2e" | "%2e." | ".%2e"
  ) {
    Cow::Owned(format!("{path}/"))
  } else {
    Cow::Borrowed(path.rfind('/').map_or("", |end| &path[..=end]))
  };
  if directory.is_empty() {
    return BTreeMap::new();
  }

  // The browser normalizes dot segments when it parses the import map.
  let prefix = if authority.is_empty()
    && !directory.starts_with('/')
    && !directory.starts_with("./")
    && !directory.starts_with("../")
  {
    "./"
  } else {
    ""
  };
  integrity_aliases(integrity, |url| {
    if url.starts_with("//") || Url::parse(url).is_ok() {
      None
    } else if url.starts_with('/') {
      Some(format!("{authority}{url}"))
    } else {
      let path = url.strip_prefix("./").unwrap_or(url);
      Some(format!("{prefix}{authority}{directory}{path}"))
    }
  })
}

fn integrity_aliases(
  integrity: &BTreeMap<String, String>,
  resolve: impl Fn(&str) -> Option<String>,
) -> BTreeMap<String, String> {
  integrity
    .iter()
    .filter_map(|(url, hash)| {
      resolve(url)
        .filter(|resolved| resolved != url)
        .map(|resolved| (resolved, hash.clone()))
    })
    .collect()
}

fn merge_existing_map(
  children: &mut [Child],
  integrity: &BTreeMap<String, String>,
  base_integrity: &BTreeMap<String, String>,
  seen_module_fetch: &mut bool,
) -> Result<bool> {
  for child in children {
    let Child::Element(element) = child else {
      continue;
    };
    if !is_active_html_element(element) {
      continue;
    }
    if is_script(element, "importmap") {
      if *seen_module_fetch {
        return Err(error!(
          "Place the import map before module scripts and modulepreload links to enable Subresource Integrity."
        ));
      }
      let mut source = String::new();
      for child in &element.children {
        if let Child::Text(text) = child {
          source.push_str(&text.data);
        }
      }
      let mut map: Map<String, Value> = serde_json::from_str(&source)
        .map_err(|error| error!("Cannot add Subresource Integrity to the import map: {error}"))?;
      let entries = map
        .entry("integrity")
        .or_insert_with(|| Value::Object(Map::new()))
        .as_object_mut()
        .ok_or_else(|| error!("The import map integrity field must be an object."))?;
      let authored = std::mem::take(entries);
      // The browser applies the last entry for a resolved URL. Keep the
      // fallback URLs first, the URLs using the late base next, and the
      // authored entries last in their original order.
      entries.extend(
        integrity
          .iter()
          .filter(|(url, _)| !base_integrity.contains_key(*url))
          .chain(base_integrity)
          .filter(|(url, _)| !authored.contains_key(*url))
          .map(|(url, hash)| (url.clone(), Value::String(hash.clone()))),
      );
      entries.extend(authored);
      element.children = vec![Child::Text(Text {
        span: DUMMY_SP,
        data: serialize_map(&Value::Object(map))?.into(),
        raw: None,
      })];
      return Ok(true);
    }
    *seen_module_fetch |= starts_module_fetch(element);
    if merge_existing_map(
      &mut element.children,
      integrity,
      base_integrity,
      seen_module_fetch,
    )? {
      return Ok(true);
    }
  }
  Ok(false)
}

fn insert_before_module_fetch(
  children: &mut Vec<Child>,
  contents: &str,
  nonce: Option<&str>,
) -> bool {
  for index in 0..children.len() {
    let Child::Element(element) = &mut children[index] else {
      continue;
    };
    if !is_active_html_element(element) {
      continue;
    }
    if starts_module_fetch(element) {
      let mut attributes = vec![HtmlPluginAttribute {
        attr_name: "type".into(),
        attr_value: Some("importmap".into()),
      }];
      if let Some(nonce) = nonce {
        attributes.push(HtmlPluginAttribute {
          attr_name: "nonce".into(),
          attr_value: Some(nonce.into()),
        });
      }
      children.insert(
        index,
        Child::Element(
          HtmlPluginTag {
            tag_name: "script".into(),
            attributes,
            inner_html: Some(contents.into()),
            ..Default::default()
          }
          .into(),
        ),
      );
      return true;
    }
    if insert_before_module_fetch(&mut element.children, contents, nonce) {
      return true;
    }
  }
  false
}

fn serialize_map(map: &Value) -> Result<String> {
  let contents = serde_json::to_string(map).to_rspack_result()?;
  // A script element is raw text, so JSON string escaping alone does not protect its end tag.
  Ok(match contents.cow_replace('<', "\\u003c") {
    Cow::Borrowed(_) => contents,
    Cow::Owned(escaped) => escaped,
  })
}
