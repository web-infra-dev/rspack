use std::borrow::Cow;

use cow_utils::CowUtils;
use rspack_error::{Result, ToStringResultToRspackResultExt, error};
use serde_json::{Map, Value};
use swc_core::common::DUMMY_SP;
use swc_html::ast::{Child, Element, Namespace, Text};
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{ExprData, Lang, NodeId, NodeKindData, SourceType};
use swc_next_ecma_parser::{CommentMode, NoTokenParserConfig, Options, Parser};
use url::Url;

use crate::{
  HtmlModuleScript,
  parser::CompiledDocument,
  tag::{HtmlPluginAttribute, HtmlPluginTag},
};

enum Placement<'a> {
  Merge(usize),
  Insert {
    index: usize,
    after: bool,
    nonce: Option<&'a str>,
  },
}

enum ImportMapUpdate {
  Merge(Map<String, Value>),
  Insert { tag: HtmlPluginTag, after: bool },
}

pub(crate) fn add_integrity_to_import_map(
  document: &mut CompiledDocument,
  integrity: Map<String, Value>,
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
  let (mut position, update) = match placement {
    Placement::Merge(position) => (position, ImportMapUpdate::Merge(integrity)),
    Placement::Insert {
      index,
      after,
      nonce,
    } => {
      let mut map = Map::new();
      map.insert("integrity".into(), Value::Object(integrity));
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
        inner_html: Some(serialize_map(&Value::Object(map))?),
        ..Default::default()
      };
      (index, ImportMapUpdate::Insert { tag, after })
    }
  };
  apply_at(children, &mut position, &mut Some(update))?;
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

fn active_elements(children: &[Child]) -> impl Iterator<Item = &Element> {
  let mut pending = vec![children.iter()];
  std::iter::from_fn(move || {
    while let Some(children) = pending.last_mut() {
      match children.next() {
        Some(Child::Element(element)) if is_active_html_element(element) => {
          pending.push(element.children.iter());
          return Some(element);
        }
        Some(_) => {}
        None => {
          pending.pop();
        }
      }
    }
    None
  })
}

#[inline(never)]
pub(crate) fn collect_module_scripts<'a>(
  document: &'a CompiledDocument,
  head_tags: &'a [HtmlPluginTag],
  body_tags: &'a [HtmlPluginTag],
) -> Vec<HtmlModuleScript<'a>> {
  let children = match document {
    CompiledDocument::Document(document) | CompiledDocument::DocumentWithoutDoctype(document) => {
      &document.children
    }
    CompiledDocument::DocumentFragment(fragment) => &fragment.children,
  };
  let mut scripts = Vec::new();
  for element in active_elements(children) {
    if !is_script(element, "module") {
      continue;
    }
    let Some(src) = attribute(element, "src") else {
      continue;
    };
    let mut asset = None;
    if element.span.is_dummy() {
      for tag in head_tags.iter().chain(body_tags) {
        if tag.tag_name == "script"
          && let Some(original) = tag.asset.as_deref()
          && tag
            .attributes
            .iter()
            .any(|attr| attr.attr_name == "src" && attr.attr_value.as_deref() == Some(src))
        {
          asset = Some(original);
          break;
        }
      }
    }
    scripts.push(HtmlModuleScript {
      src: src.trim(),
      asset,
      has_integrity: attribute(element, "integrity").is_some_and(|value| !value.trim().is_empty()),
    });
  }
  scripts
}

struct ModuleUrl<'a> {
  input: &'a str,
  url: Url,
  absolute: bool,
  has_authority: bool,
  path_relative: bool,
  climbs: bool,
}

impl<'a> ModuleUrl<'a> {
  fn parse(input: &'a str, root: &Url, document: &Url) -> Option<Self> {
    let input = input.trim();
    let url = root.join(input).ok()?;
    let absolute = Url::parse(input).is_ok();
    let starts_with_slash = input.starts_with(['/', '\\']);
    let has_authority = absolute || (starts_with_slash && input[1..].starts_with(['/', '\\']));
    let path_relative = !absolute && !starts_with_slash;
    // An unknown deployment directory can affect comparisons involving `..`.
    let climbs = path_relative
      && document.join(input).ok().is_some_and(|joined| {
        joined
          .path()
          .strip_prefix(document.path().trim_end_matches('/'))
          != Some(url.path())
      });
    Some(Self {
      input,
      url,
      absolute,
      has_authority,
      path_relative,
      climbs,
    })
  }

  fn could_match(&self, other: &ModuleUrl<'_>) -> bool {
    let path = self.url.path();
    let other_path = other.url.path();
    let same_path = path_matches(
      path,
      other_path,
      self.path_relative && (!other.path_relative || self.climbs || other.climbs),
    ) || (other.path_relative
      && (!self.path_relative || self.climbs || other.climbs)
      && path_matches(other_path, path, true));
    same_path && (!self.has_authority || !other.has_authority || self.could_share_origin(other))
  }

  fn could_share_origin(&self, other: &ModuleUrl<'_>) -> bool {
    if self.url.host_str() != other.url.host_str() {
      return false;
    }
    match (self.absolute, other.absolute) {
      (true, true) => self.url.origin() == other.url.origin(),
      (true, false) => self
        .url
        .join(other.input)
        .is_ok_and(|url| url.origin() == self.url.origin()),
      (false, true) => other
        .url
        .join(self.input)
        .is_ok_and(|url| url.origin() == other.url.origin()),
      (false, false) => {
        // Scheme-relative URLs inherit the page's scheme, which also determines the default port.
        self.url.origin() == other.url.origin() || {
          let http = Url::parse("http://rspack.invalid/").expect("valid URL");
          http.join(self.input).ok().map(|url| url.origin())
            == http.join(other.input).ok().map(|url| url.origin())
        }
      }
    }
  }

  fn changes_base_for(&self, key: &ModuleUrl<'_>) -> bool {
    if key.absolute {
      return false;
    }
    if key.has_authority {
      return self.absolute;
    }
    self.has_authority
      || (key.path_relative
        && (!self.path_relative
          || self.climbs
          || !self
            .url
            .path()
            .rsplit_once('/')
            .is_some_and(|(directory, _)| directory.is_empty())))
  }
}

fn path_matches(path: &str, other: &str, allow_suffix: bool) -> bool {
  let path = path.strip_prefix('/').unwrap_or(path);
  let other = other.strip_prefix('/').unwrap_or(other);
  let mut other_segments = other.rsplit('/');
  // HTML encodes asset names segment by segment and can append a cache-busting query.
  path.rsplit('/').all(|segment| {
    other_segments.next().is_some_and(|other| {
      segment == other
        || (segment.contains('%') || other.contains('%'))
          && urlencoding::decode(segment)
            .ok()
            .zip(urlencoding::decode(other).ok())
            .is_some_and(|(segment, other)| segment == other)
    })
  }) && (allow_suffix || other_segments.next().is_none())
}

#[derive(PartialEq)]
enum InlineModuleImports {
  None,
  Deferred { uses_base: bool },
  Immediate,
}

fn inline_module_imports(
  element: &Element,
  mut deferred_absolute: Option<&mut Vec<Url>>,
) -> InlineModuleImports {
  let mut source = String::new();
  for child in &element.children {
    if let Child::Text(text) = child {
      source.push_str(&text.data);
    }
  }
  if !source.contains("import") && !source.contains("export") {
    return InlineModuleImports::None;
  }
  let allocator = Allocator::new();
  let parsed = Parser::init(
    &allocator,
    &source,
    Options {
      source_type: SourceType::Module,
      lang: Lang::Js,
      preserve_parens: false,
      comments: CommentMode::None,
    },
    NoTokenParserConfig,
  )
  .parse();
  if !parsed.diagnostics.is_empty() {
    return InlineModuleImports::Immediate;
  }
  // SAFETY: The scan only reads nodes from this unmodified parser output.
  let ast = unsafe { parsed.ast.ast() };
  // Non-async inline modules run after parsing. Static dependencies can fetch
  // during parsing, while import() starts fetching when the module executes.
  let is_async = element.attributes.iter().any(|attr| attr.name == "async");
  let mut imports = InlineModuleImports::None;
  for index in 0..ast.node_count() {
    match ast.kind_data(NodeId::from_raw_unchecked(index as u32)) {
      NodeKindData::ImportDeclaration(_) | NodeKindData::ExportAllDeclaration(_) => {
        return InlineModuleImports::Immediate;
      }
      NodeKindData::ExportNamedDeclaration(export) if export.source(ast).is_some() => {
        return InlineModuleImports::Immediate;
      }
      NodeKindData::ImportExpression(import) => {
        if is_async {
          return InlineModuleImports::Immediate;
        }
        let absolute = match ast.expr_data(import.source(ast)) {
          ExprData::StringLiteral(literal) => ast
            .get_wtf8(literal.value(ast))
            .as_str()
            .and_then(|specifier| Url::parse(specifier).ok()),
          _ => None,
        };
        imports = InlineModuleImports::Deferred {
          uses_base: absolute.is_none()
            || matches!(imports, InlineModuleImports::Deferred { uses_base: true }),
        };
        if let (Some(url), Some(urls)) = (absolute, deferred_absolute.as_mut()) {
          urls.push(url);
        }
      }
      _ => {}
    }
  }
  imports
}

fn find_placement<'a>(
  children: &'a [Child],
  integrity: &Map<String, Value>,
) -> Result<Option<Placement<'a>>> {
  let mut seen_base = false;
  let mut base = None;
  let mut first_map = None;
  let mut last_map = None;
  let mut map_nonce = None;
  let mut first_fetch = None;
  let mut first_bundled_fetch = None;
  let mut first_fetch_after_map = None;
  let mut nonce = None;
  let mut bundled_nonce = None;
  let mut preload_nonce = None;
  let mut first_fetch_after_base = None;
  let mut bundled_fetch_after_base = false;
  let mut has_deferred_import = false;
  let mut deferred_import_before_base = false;
  let mut deferred_absolute_before_base = Vec::new();
  let root = Url::parse("https://rspack.invalid/").expect("valid URL");
  let document = root.join("document/").expect("valid URL");
  let urls = integrity
    .keys()
    .filter_map(|url| ModuleUrl::parse(url, &root, &document))
    .collect::<Vec<_>>();

  for (index, element) in active_elements(children).enumerate() {
    if !seen_base
      && element.tag_name == "base"
      && let Some(href) = element.attributes.iter().find(|attr| attr.name == "href")
    {
      seen_base = true;
      // A base without a path or authority cannot change the module's URL.
      if let Some(value) = href.value.as_deref().map(str::trim)
        && !value.is_empty()
        && !value.starts_with(['?', '#'])
        && let Some(url) = ModuleUrl::parse(value, &root, &document)
        && !matches!(url.url.scheme(), "data" | "javascript")
      {
        base = Some((index, url));
      }
    }

    let element_nonce = attribute(element, "nonce").filter(|value| !value.is_empty());
    if is_script(element, "importmap") {
      if first_bundled_fetch.is_some() {
        return Err(error!(
          "Place import maps before scripts and preloads of bundled modules to enable Subresource Integrity."
        ));
      }
      first_map.get_or_insert(index);
      last_map = Some(index);
      first_fetch_after_map = None;
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
    first_fetch_after_map.get_or_insert(index);

    let attr = if module_script { "src" } else { "href" };
    let source = element
      .attributes
      .iter()
      .find(|attribute| attribute.name == attr);
    let url = source
      .and_then(|attribute| attribute.value.as_deref())
      .map(str::trim)
      .filter(|url| !url.is_empty());
    let inline_imports = if module_script && source.is_none() {
      inline_module_imports(
        element,
        base.is_none().then_some(&mut deferred_absolute_before_base),
      )
    } else {
      InlineModuleImports::None
    };
    let bundled = if let Some(url) = url {
      ModuleUrl::parse(url, &root, &document).is_some_and(|url| {
        urls.iter().any(|key| {
          // Injected module scripts can have a cache-busting query from `hash: true`.
          ((module_script && element.span.is_dummy()) || url.url.query() == key.url.query())
            && url.could_match(key)
        })
      })
    } else {
      inline_imports == InlineModuleImports::Immediate
    };
    if bundled {
      first_bundled_fetch.get_or_insert(index);
      bundled_nonce = bundled_nonce.or(element_nonce);
    }
    let deferred = matches!(inline_imports, InlineModuleImports::Deferred { .. });
    has_deferred_import |= deferred;
    if base.is_some() {
      first_fetch_after_base.get_or_insert(index);
      bundled_fetch_after_base |= bundled || deferred;
    } else {
      // Relative imports resolve against the base captured by the inline module.
      deferred_import_before_base |= matches!(
        inline_imports,
        InlineModuleImports::Deferred { uses_base: true }
      );
    }
  }

  let generated_nonce = map_nonce.or(bundled_nonce).or(nonce).or(preload_nonce);
  let after_last_map = if has_deferred_import { last_map } else { None };
  // Register the integrity-only map under the browser's effective base. One
  // earlier map cannot describe both outcomes if CSP rejects a later base.
  if let Some((base_index, base)) = base
    && first_map
      .or(first_fetch)
      .is_some_and(|index| index < base_index)
    && urls.iter().any(|key| {
      base.changes_base_for(key)
        && (bundled_fetch_after_base
          || (base.has_authority
            && deferred_absolute_before_base.iter().any(|specifier| {
              specifier
                .join(base.input)
                .and_then(|base| base.join(key.input))
                .is_ok_and(|key| key == *specifier)
            })))
    })
  {
    if deferred_import_before_base || first_bundled_fetch.is_some_and(|index| index < base_index) {
      return Err(error!(
        "Place the base element before module scripts and preloads of bundled modules to enable Subresource Integrity."
      ));
    }
    let before = if last_map.is_some_and(|index| index > base_index) {
      first_fetch_after_map
    } else {
      first_fetch_after_base
    };
    let after = Some(last_map.unwrap_or(base_index).max(base_index));
    return Ok(insert_placement(before, after, generated_nonce));
  }

  if first_map == last_map
    && let Some(index) = first_map
  {
    return Ok(Some(Placement::Merge(index)));
  }

  // The browser keeps the first matching integrity rule across import maps.
  // Register generated rules after any authored maps.
  Ok(insert_placement(
    first_fetch_after_map,
    after_last_map,
    generated_nonce,
  ))
}

fn insert_placement(
  before: Option<usize>,
  after: Option<usize>,
  nonce: Option<&str>,
) -> Option<Placement<'_>> {
  let (index, after) = match before {
    Some(index) => (index, false),
    None => (after?, true),
  };
  Some(Placement::Insert {
    index,
    after,
    nonce,
  })
}

fn apply_at(
  children: &mut Vec<Child>,
  position: &mut usize,
  update: &mut Option<ImportMapUpdate>,
) -> Result<bool> {
  for index in 0..children.len() {
    let Child::Element(element) = &mut children[index] else {
      continue;
    };
    if !is_active_html_element(element) {
      continue;
    }
    if *position == 0 {
      match update.take() {
        Some(ImportMapUpdate::Insert { tag, after }) => {
          children.insert(index + usize::from(after), Child::Element(tag.into()));
        }
        Some(ImportMapUpdate::Merge(integrity)) => merge_integrity(element, integrity)?,
        None => return Ok(false),
      }
      return Ok(true);
    }
    *position -= 1;
    if apply_at(&mut element.children, position, update)? {
      return Ok(true);
    }
  }
  Ok(false)
}

fn merge_integrity(element: &mut Element, mut integrity: Map<String, Value>) -> Result<()> {
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
  // The browser applies the last entry for a resolved URL within one map.
  integrity.retain(|url, _| !entries.contains_key(url));
  integrity.extend(std::mem::take(entries));
  *entries = integrity;
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
