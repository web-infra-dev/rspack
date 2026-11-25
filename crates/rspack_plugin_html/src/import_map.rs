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

enum Placement<'a> {
  Merge(usize),
  Insert(usize, Option<&'a str>),
}

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
  let Some(placement) = find_placement(children, &integrity)? else {
    return Ok(());
  };
  let (mut position, mut tag) = match placement {
    Placement::Merge(position) => (position, None),
    Placement::Insert(position, nonce) => {
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
      let tag = HtmlPluginTag {
        tag_name: "script".into(),
        attributes,
        inner_html: Some(serialize_map(&json!({ "integrity": integrity }))?),
        ..Default::default()
      };
      (position, Some(tag))
    }
  };
  apply_at(children, &mut position, &mut tag, &integrity)?;
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

fn is_modulepreload(element: &Element) -> bool {
  element.tag_name == "link"
    && attribute(element, "rel").is_some_and(|value| {
      value
        .split_ascii_whitespace()
        .any(|rel| rel.eq_ignore_ascii_case("modulepreload"))
    })
}

fn is_active_html_element(element: &Element) -> bool {
  element.namespace == Namespace::HTML && !matches!(&*element.tag_name, "template" | "noscript")
}

fn asset_basename(url: &str) -> Cow<'_, str> {
  let name = url
    .split(['?', '#'])
    .next()
    .unwrap_or_default()
    .rsplit(['/', '\\'])
    .next()
    .unwrap_or_default();
  urlencoding::decode(name).unwrap_or(Cow::Borrowed(name))
}

fn find_placement<'a>(
  children: &'a [Child],
  integrity: &BTreeMap<String, String>,
) -> Result<Option<Placement<'a>>> {
  let mut pending = vec![children.iter()];
  let mut position = 0;
  let mut seen_base = false;
  let mut base = None;
  let mut first_map = None;
  let mut multiple_maps = false;
  let mut map_nonce = None;
  let mut first_fetch = None;
  let mut nonce = None;
  let mut preload_nonce = None;
  let mut before_base = false;
  let mut first_fetch_after_base = None;
  let mut relative_fetch_after_base = false;

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
    let index = position;
    position += 1;

    if !seen_base
      && element.tag_name == "base"
      && let Some(href) = element.attributes.iter().find(|attr| attr.name == "href")
    {
      seen_base = true;
      // A base without a path or authority cannot change the module's URL.
      if href.value.as_deref().is_some_and(|value| {
        let value = value.trim();
        !value.is_empty() && !value.starts_with(['?', '#'])
      }) {
        base = Some(index);
      }
    }

    let element_nonce = attribute(element, "nonce").filter(|value| !value.is_empty());
    if is_script(element, "importmap") {
      if first_fetch.is_some() {
        return Err(error!(
          "Place import maps before module scripts and modulepreload links to enable Subresource Integrity."
        ));
      }
      multiple_maps |= first_map.is_some();
      first_map.get_or_insert(index);
      map_nonce = element_nonce.or(map_nonce);
      continue;
    }

    let module_script = is_script(element, "module");
    if !module_script && !is_modulepreload(element) {
      continue;
    }

    if let Some(value) = element_nonce {
      if first_fetch.is_none() || module_script {
        nonce.get_or_insert(value);
      } else {
        preload_nonce.get_or_insert(value);
      }
    }
    first_fetch.get_or_insert(index);

    let url = attribute(element, if module_script { "src" } else { "href" });
    if base.is_some() {
      first_fetch_after_base.get_or_insert(index);
      relative_fetch_after_base |= url.map_or(module_script, |url| {
        matches!(Url::parse(url), Err(ParseError::RelativeUrlWithoutBase))
      });
    } else {
      before_base |= module_script
        || url.is_some_and(|url| {
          let basename = asset_basename(url);
          integrity.keys().any(|url| asset_basename(url) == basename)
        });
    }
  }

  let generated_nonce = map_nonce.or(nonce).or(preload_nonce);
  // Register the integrity-only map under the browser's effective base. One
  // earlier map cannot describe both outcomes if CSP rejects a later base.
  if let (Some(base), Some(after_base)) = (base, first_fetch_after_base)
    && first_map.or(first_fetch).is_some_and(|index| index < base)
    && relative_fetch_after_base
    && integrity.keys().any(|url| url.starts_with(['.', '/']))
  {
    if before_base {
      return Err(error!(
        "Place the base element before module scripts and preloads of bundled modules to enable Subresource Integrity."
      ));
    }
    return Ok(Some(Placement::Insert(after_base, generated_nonce)));
  }

  if !multiple_maps && let Some(index) = first_map {
    return Ok(Some(Placement::Merge(index)));
  }

  // The browser keeps the first matching integrity rule across import maps.
  // Register generated rules after any authored maps.
  Ok(first_fetch.map(|index| Placement::Insert(index, generated_nonce)))
}

fn apply_at(
  children: &mut Vec<Child>,
  position: &mut usize,
  tag: &mut Option<HtmlPluginTag>,
  integrity: &BTreeMap<String, String>,
) -> Result<bool> {
  for index in 0..children.len() {
    let Child::Element(element) = &mut children[index] else {
      continue;
    };
    if !is_active_html_element(element) {
      continue;
    }
    if *position == 0 {
      if let Some(tag) = tag.take() {
        children.insert(index, Child::Element(tag.into()));
      } else {
        merge_integrity(element, integrity)?;
      }
      return Ok(true);
    }
    *position -= 1;
    if apply_at(&mut element.children, position, tag, integrity)? {
      return Ok(true);
    }
  }
  Ok(false)
}

fn merge_integrity(element: &mut Element, integrity: &BTreeMap<String, String>) -> Result<()> {
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
  // The browser applies the last entry for a resolved URL within one map.
  entries.extend(
    integrity
      .iter()
      .filter(|(url, _)| !authored.contains_key(*url))
      .map(|(url, hash)| (url.clone(), Value::String(hash.clone()))),
  );
  entries.extend(authored);
  element.children = vec![Child::Text(Text {
    span: DUMMY_SP,
    data: serialize_map(&Value::Object(map))?.into(),
    raw: None,
  })];
  Ok(())
}

fn serialize_map(map: &Value) -> Result<String> {
  let contents = serde_json::to_string(map).to_rspack_result()?;
  // A script element is raw text, so JSON string escaping alone does not protect its end tag.
  Ok(match contents.cow_replace('<', "\\u003c") {
    Cow::Borrowed(_) => contents,
    Cow::Owned(escaped) => escaped,
  })
}
