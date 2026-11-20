use std::{borrow::Cow, iter, sync::Arc};

use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_collections::{Identifiable, Identifier};
use rspack_error::{Result, impl_empty_diagnosable_trait};
use rspack_hash::{RspackHashDigest, RspackHasher};
use rspack_hook::define_hook;
use rspack_macros::impl_source_map_config;
use rspack_util::{json_stringify_str, source_map::SourceMapKind};
use rustc_hash::FxHashMap as HashMap;
use serde::Serialize;

use crate::{
  BoxChunkInitFragment, BoxModule, BuildContext, BuildInfo, BuildMeta, BuildMetaDefaultObject,
  BuildMetaExportsType, ChunkGraph, ChunkInitFragments, ChunkUkey,
  CodeGenerationDataChunkInitFragments, CodeGenerationDataUrl, CodeGenerationResultBuilder,
  Compilation, ConcatenationScope, Context, CssLayer, CssModuleRenderCondition, DependenciesBlock,
  DependenciesBlockData, DependencyRef, ExportProvided, ExternalType, FactoryMetaStore, FreezeLock,
  ImportAttributes, ImportPhase, InitFragmentExt, InitFragmentKey, InitFragmentStage,
  LibIdentOptions, Module, ModuleArgument, ModuleCodeGenerationContext, ModuleCodeTemplate,
  ModuleGraph, ModuleType, NAMESPACE_OBJECT_EXPORT, NeedBuildContext, NormalInitFragment,
  RuntimeGlobals, RuntimeSpec, SourceType, StaticExportsDependency, StaticExportsSpec, UsageState,
  UsedExports, UsedNameItem, css_module_render_conditions_identifier, extract_url_and_global,
  impl_module_meta_info, module_update_hash, property_access,
  rspack_sources::{BoxSource, RawStringSource, SourceExt},
  to_identifier,
};

static EXTERNAL_MODULE_JS_SOURCE_TYPES: &[SourceType] = &[SourceType::JavaScript];
static EXTERNAL_MODULE_CSS_SOURCE_TYPES: &[SourceType] = &[SourceType::CssImport];
static EXTERNAL_MODULE_ASSET_URL_SOURCE_TYPES: &[SourceType] = &[SourceType::AssetUrl];

#[cacheable]
#[derive(Debug, Clone, Copy)]
pub enum ExternalInterop {
  Default,
  EsModule,
}

impl ExternalInterop {
  pub fn as_str(self) -> &'static str {
    match self {
      Self::Default => "default",
      Self::EsModule => "esModule",
    }
  }
}

define_hook!(ExternalModuleChunkCondition: SeriesBail(
  chunk_ukey: &ChunkUkey,
  compilation: &Compilation
) -> bool,tracing=false);

#[cacheable]
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum ExternalRequest {
  Single(ExternalRequestValue),
  Map(HashMap<String, ExternalRequestValue>),
}

#[cacheable]
#[derive(Debug, Clone)]
pub struct ExternalRequestValue {
  pub primary: String,
  rest: Option<Vec<String>>,
}

impl ExternalRequestValue {
  pub fn has_rest(&self) -> bool {
    self.rest.as_ref().is_some_and(|r| !r.is_empty())
  }
}

/// The CommonJS require form used to render an external request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommonJsExternalRequireKind {
  CommonJs,
  NodeCommonJs,
}

impl CommonJsExternalRequireKind {
  pub fn from_external_type(external_type: &str) -> Option<Self> {
    match external_type {
      "commonjs" | "commonjs2" | "commonjs-module" | "commonjs-static" => Some(Self::CommonJs),
      "node-commonjs" => Some(Self::NodeCommonJs),
      _ => None,
    }
  }

  /// Renders the complete require expression and installs any init fragment
  /// required by its callee.
  pub fn render_expression<S: AsRef<str>>(
    self,
    request: Option<&str>,
    properties: impl IntoIterator<Item = S>,
    compilation: &Compilation,
    chunk_init_fragments: &mut ChunkInitFragments,
  ) -> String {
    let require = self.render_callee(compilation, chunk_init_fragments);
    format!(
      "{require}({}){}",
      request.map_or_else(|| "undefined".to_string(), json_stringify_str),
      property_access(properties, 0)
    )
  }

  /// Renders only the require callee and installs its required init fragment.
  ///
  /// This is the split-range counterpart of [`Self::render_expression`].
  pub fn render_callee(
    self,
    compilation: &Compilation,
    chunk_init_fragments: &mut ChunkInitFragments,
  ) -> &'static str {
    match self {
      Self::NodeCommonJs if compilation.options.output.module => {
        chunk_init_fragments.push(create_node_commonjs_init_fragment(compilation));
        "__rspack_createRequire_require"
      }
      Self::CommonJs | Self::NodeCommonJs => "require",
    }
  }
}

impl Serialize for ExternalRequestValue {
  fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    if self.rest.is_none() {
      self.primary.serialize(serializer)
    } else {
      self.iter().collect::<Vec<_>>().serialize(serializer)
    }
  }
}

impl ExternalRequestValue {
  pub fn new(primary: String, rest: Option<Vec<String>>) -> Self {
    Self { primary, rest }
  }

  pub fn primary(&self) -> &str {
    &self.primary
  }

  pub fn rest(&self) -> Option<&[String]> {
    self.rest.as_deref()
  }

  pub fn iter(&self) -> impl Iterator<Item = &String> {
    if let Some(rest) = &self.rest {
      iter::once(&self.primary).chain(rest)
    } else {
      iter::once(&self.primary).chain(&[])
    }
  }
}

fn get_namespace_object_export(
  concatenation_scope: Option<&mut ConcatenationScope>,
  supports_const: bool,
  runtime_template: &mut ModuleCodeTemplate,
) -> String {
  if let Some(concatenation_scope) = concatenation_scope {
    concatenation_scope.register_namespace_export(NAMESPACE_OBJECT_EXPORT);
    format!(
      "{} {NAMESPACE_OBJECT_EXPORT}",
      if supports_const { "const" } else { "var" }
    )
  } else {
    format!(
      "{}.exports",
      runtime_template.render_module_argument(ModuleArgument::Module)
    )
  }
}

fn get_source_for_global_variable_external(
  variable_names: Option<&ExternalRequestValue>,
  external_type: &ExternalType,
) -> String {
  let object_lookup = if let Some(variable_names) = variable_names {
    property_access(variable_names.iter(), 0)
  } else {
    "[undefined]".to_string()
  };
  format!("{external_type}{object_lookup}")
}

fn get_request_string(request: &ExternalRequestValue) -> String {
  let variable_name = request.primary();
  let object_lookup = property_access(request.iter(), 1);
  format!("{variable_name}{object_lookup}")
}

fn create_node_commonjs_init_fragment(compilation: &Compilation) -> BoxChunkInitFragment {
  let need_prefix = compilation
    .options
    .output
    .environment
    .supports_node_prefix_for_core_modules();

  NormalInitFragment::new(
    format!(
      "import {{ createRequire as __rspack_createRequire }} from \"{}\";\n{} __rspack_createRequire_require = __rspack_createRequire({}.url);\n",
      if need_prefix { "node:module" } else { "module" },
      if compilation.options.output.environment.supports_const() {
        "const"
      } else {
        "var"
      },
      compilation.options.output.import_meta_name
    ),
    InitFragmentStage::StageESMImports,
    0,
    InitFragmentKey::ModuleExternal("node-commonjs".to_string()),
    None,
  )
  .with_top_level_decl_symbols(vec![
    "__rspack_createRequire".into(),
    "__rspack_createRequire_require".into(),
  ])
  .boxed()
}

fn get_source_for_import(
  module_and_specifiers: Option<&ExternalRequestValue>,
  compilation: &Compilation,
  attributes: &Option<ImportAttributes>,
  phase: ImportPhase,
) -> String {
  let import_function = if phase != ImportPhase::Evaluation {
    format!(
      "{}.{}",
      compilation.options.output.import_function_name,
      phase.as_str()
    )
  } else {
    compilation.options.output.import_function_name.clone()
  };

  format!(
    "{}({}).then(function(module) {{ return module{}; }})",
    import_function,
    {
      let attributes_str = if let Some(attributes) = attributes {
        format!(
          ", {{ with: {} }}",
          simd_json::to_string(attributes).expect("invalid json to_string")
        )
      } else {
        String::new()
      };

      format!(
        "{}{}",
        if let Some(module_and_specifiers) = module_and_specifiers {
          rspack_util::json_stringify_str(module_and_specifiers.primary())
        } else {
          "undefined".to_string()
        },
        attributes_str
      )
    },
    if let Some(module_and_specifiers) = module_and_specifiers {
      property_access(module_and_specifiers.iter(), 1)
    } else {
      String::new()
    }
  )
}

fn module_external_fragment_key(
  base: &str,
  attributes: &Option<ImportAttributes>,
  phase: ImportPhase,
) -> String {
  let phase_str = if phase == ImportPhase::Evaluation {
    String::new()
  } else {
    format!("|phase={}", phase.as_str())
  };
  if let Some(attributes) = attributes {
    format!(
      "{}{}|{}",
      base,
      phase_str,
      simd_json::to_string(attributes).expect("json stringify failed")
    )
  } else {
    format!("{base}{phase_str}")
  }
}

fn module_external_import_statement(
  module_and_specifiers: &ExternalRequestValue,
  ident: &str,
  attributes: &Option<ImportAttributes>,
  phase: ImportPhase,
) -> String {
  let request = json_stringify_str(module_and_specifiers.primary());
  let mut import_statement = String::from("import");
  if phase != ImportPhase::Evaluation {
    import_statement.push(' ');
    import_statement.push_str(phase.as_str());
  }
  import_statement.push(' ');
  if !phase.is_source() {
    import_statement.push_str("* as ");
  }
  import_statement.push_str("__rspack_external_");
  import_statement.push_str(ident);
  import_statement.push_str(" from ");
  import_statement.push_str(&request);
  if let Some(attributes) = attributes {
    import_statement.push_str(" with ");
    import_statement.push_str(&simd_json::to_string(attributes).expect("json stringify failed"));
  }
  import_statement.push_str(";\n");
  import_statement
}

#[derive(Debug, PartialEq, Eq)]
struct ModuleExternalRemapping {
  exposed_name: String,
  raw_export_name: String,
  nested: Option<Vec<ModuleExternalRemapping>>,
}

fn collect_module_external_remapping(
  exports_info_artifact: &crate::ExportsInfoArtifact,
  exports_info: &crate::ExportsInfoData,
  runtime: Option<&RuntimeSpec>,
) -> Option<Vec<ModuleExternalRemapping>> {
  if exports_info.other_exports_info().get_used(runtime) != UsageState::Unused {
    return None;
  }

  let remapping: Vec<_> = exports_info
    .exports()
    .iter()
    .filter(|(_, export_info)| !matches!(export_info.provided(), Some(ExportProvided::NotProvided)))
    .filter_map(|(export_name, export_info)| {
      let UsedNameItem::Str(used_name) = export_info.get_used_name(Some(export_name), runtime)?
      else {
        return None;
      };

      let nested = if export_info.get_used(runtime) == UsageState::OnlyPropertiesUsed {
        export_info.exports_info().and_then(|nested_exports_info| {
          collect_module_external_remapping(
            exports_info_artifact,
            nested_exports_info.as_data(exports_info_artifact),
            runtime,
          )
        })
      } else {
        None
      };

      Some(ModuleExternalRemapping {
        exposed_name: used_name.to_string(),
        raw_export_name: export_name.to_string(),
        nested,
      })
    })
    .collect();

  (!remapping
    .iter()
    .all(|item| item.exposed_name == item.raw_export_name && item.nested.is_none()))
  .then_some(remapping)
}

fn render_module_external_remapping(
  input: &str,
  remapping: &[ModuleExternalRemapping],
  runtime_template: &mut ModuleCodeTemplate,
  create_namespace_object_name: &str,
  wrap_namespace_getter_name: &str,
) -> String {
  let properties = remapping
    .iter()
    .map(|remapping| {
      let access = format!(
        "{input}{}",
        property_access([remapping.raw_export_name.as_str()], 0)
      );
      let getter = if let Some(nested) = &remapping.nested {
        format!(
          "{wrap_namespace_getter_name}({})",
          render_module_external_remapping(
            &access,
            nested,
            runtime_template,
            create_namespace_object_name,
            wrap_namespace_getter_name,
          )
        )
      } else {
        runtime_template.returning_function(&access, "")
      };

      format!(
        "{}: {getter}",
        module_external_remapping_property_key(&remapping.exposed_name)
      )
    })
    .collect::<Vec<_>>()
    .join(", ");

  format!("{create_namespace_object_name}({{{properties}}})")
}

fn module_external_remapping_property_key(exposed_name: &str) -> String {
  format!("[{}]", json_stringify_str(exposed_name))
}

fn get_source_for_module_external(
  module_and_specifiers: &ExternalRequestValue,
  ident: &str,
  dependency_meta: &DependencyMeta,
  exports_info_artifact: &crate::ExportsInfoArtifact,
  exports_info: &crate::ExportsInfoData,
  runtime: Option<&RuntimeSpec>,
  runtime_template: &mut ModuleCodeTemplate,
) -> (Option<String>, String, ChunkInitFragments) {
  let external_module_id = format!("__rspack_external_{ident}");
  let fragment = NormalInitFragment::new(
    module_external_import_statement(
      module_and_specifiers,
      ident,
      &dependency_meta.attributes,
      dependency_meta.phase,
    ),
    InitFragmentStage::StageESMImports,
    0,
    InitFragmentKey::ModuleExternal(module_external_fragment_key(
      module_and_specifiers.primary(),
      &dependency_meta.attributes,
      dependency_meta.phase,
    )),
    None,
  )
  .with_top_level_decl_symbols(vec![external_module_id.clone().into()]);
  let chunk_init_fragments: ChunkInitFragments = vec![fragment.boxed()];

  let base_access = format!(
    "{external_module_id}{}",
    property_access(module_and_specifiers.iter(), 1)
  );
  let remapping = collect_module_external_remapping(exports_info_artifact, exports_info, runtime);
  let create_namespace_object_name = format!("__rspack_create_module_external_namespace_{ident}");
  let wrap_namespace_getter_name = format!("__rspack_wrap_module_external_namespace_{ident}");
  let expression = if let Some(remapping) = remapping.as_ref() {
    render_module_external_remapping(
      &base_access,
      remapping,
      runtime_template,
      &create_namespace_object_name,
      &wrap_namespace_getter_name,
    )
  } else {
    base_access
  };
  let init = remapping.map(|_| {
    let define_property_getters =
      runtime_template.render_runtime_globals(&RuntimeGlobals::DEFINE_PROPERTY_GETTERS);
    let create_namespace_object = runtime_template.basic_function(
      "y",
      &format!("var x = {{}};\n{define_property_getters}(x, y);\nreturn x;"),
    );
    let return_x = runtime_template.returning_function("x", "");
    format!(
      "var {create_namespace_object_name} = {create_namespace_object};\nvar {wrap_namespace_getter_name} = {};",
      runtime_template.returning_function(&return_x, "x")
    )
  });

  (init, expression, chunk_init_fragments)
}

/**
 * Resolve the detailed external type from the raw external type.
 * e.g. resolve "module" or "import" from "module-import" type
 */
fn resolve_external_type<'a>(
  external_type: &'a str,
  dependency_meta: &'a DependencyMeta,
) -> &'a str {
  match external_type {
    "commonjs-import" => {
      if let Some(ExternalTypeEnum::Import) = dependency_meta.external_type.as_ref() {
        "import"
      } else {
        "commonjs"
      }
    }
    "module-import" => {
      if matches!(
        dependency_meta.external_type.as_ref(),
        Some(ExternalTypeEnum::Import)
      ) {
        "import"
      } else {
        "module"
      }
    }
    "modern-module" => {
      if let Some(external_type) = dependency_meta.external_type.as_ref() {
        match external_type {
          ExternalTypeEnum::Import => "import",
          ExternalTypeEnum::Module => "module",
          ExternalTypeEnum::CommonJs(external_type) => external_type.as_str(),
        }
      } else {
        "module"
      }
    }
    "asset" | "asset-url" => {
      if dependency_meta.source_type == Some(SourceType::AssetUrl) {
        "asset-url"
      } else {
        "asset"
      }
    }

    import_or_module => import_or_module,
  }
}

#[impl_source_map_config]
#[cacheable]
#[derive(Debug)]
pub struct ExternalModule {
  dependencies_block: DependenciesBlockData,
  pub id: Identifier,
  pub request: ExternalRequest,
  pub external_type: ExternalType,
  /// Request intended by user (without loaders from config)
  user_request: String,
  factory_meta: FactoryMetaStore,
  build_info: FreezeLock<BuildInfo>,
  build_meta: FreezeLock<BuildMeta>,
  dependency_meta: DependencyMeta,
  place_in_initial: bool,
  interop: Option<ExternalInterop>,
}

#[cacheable]
#[derive(Debug)]
pub enum ExternalTypeEnum {
  Import,
  Module,
  CommonJs(ExternalType),
}

pub type MetaExternalType = Option<ExternalTypeEnum>;

#[cacheable]
#[derive(Debug)]
pub struct DependencyMeta {
  pub external_type: MetaExternalType,
  pub attributes: Option<ImportAttributes>,
  pub phase: ImportPhase,
  pub source_type: Option<SourceType>,
  pub css_import_conditions: Vec<CssModuleRenderCondition>,
}

#[derive(Debug, Clone)]
struct MediaType {
  decoded: String,
  output: String,
}

impl MediaType {
  fn from_ident(ident: &ParsedIdent<'_>) -> Self {
    let decoded = ident.decoded.as_ref();
    let output = if is_plain_ident(decoded) {
      decoded.to_string()
    } else {
      ident.raw.to_string()
    };
    Self {
      decoded: decoded.to_string(),
      output,
    }
  }

  fn from_plain(name: &str) -> Self {
    Self {
      decoded: name.to_string(),
      output: name.to_string(),
    }
  }

  fn eq_ignore_ascii_case(&self, other: &str) -> bool {
    self.decoded.eq_ignore_ascii_case(other)
  }
}

struct ParsedIdent<'a> {
  decoded: Cow<'a, str>,
  raw: &'a str,
}

#[derive(Debug, Clone)]
enum ParsedMediaQuery {
  Never,
  Positive {
    only: bool,
    media_type: Option<MediaType>,
    condition: Option<String>,
  },
  Negated {
    media_type: MediaType,
    condition: Option<String>,
  },
}

/// Renders an `@import` rule for an external stylesheet.
///
/// Conditions are ordered from the outermost to the innermost context and are
/// folded into the `@import` prelude: nested layers become a single layer path,
/// supports conditions are combined with `and`, and media query lists are
/// intersected.
fn render_css_import_source<'a>(
  request: &str,
  conditions: impl IntoIterator<Item = &'a CssModuleRenderCondition>,
) -> String {
  let mut layer: Option<CssLayer> = None;
  let mut supports: Vec<&str> = Vec::new();
  let mut media: Option<String> = None;

  for condition in conditions {
    if let Some(condition_layer) = &condition.layer {
      layer = Some(match (layer.take(), condition_layer.clone()) {
        (None, layer) => layer,
        (Some(outer), inner) => merge_css_layers(outer, inner),
      });
    }
    if let Some(value) = condition
      .supports
      .as_deref()
      .filter(|value| !value.is_empty())
    {
      supports.push(value);
    }
    if let Some(value) = condition.media.as_deref().filter(|value| !value.is_empty()) {
      media = Some(match media {
        Some(outer) => merge_media_query_lists(&outer, value),
        None => value.to_string(),
      });
    }
  }

  let mut source = format!("@import url({})", json_stringify_str(request));
  match layer {
    Some(CssLayer::Anonymous) => source.push_str(" layer()"),
    Some(CssLayer::Named(layer)) => {
      source.push_str(" layer(");
      source.push_str(&layer);
      source.push(')');
    }
    None => {}
  }
  if let Some((first, rest)) = supports.split_first() {
    source.push_str(" supports(");
    if rest.is_empty() {
      source.push_str(first);
    } else {
      source.push('(');
      source.push_str(first);
      source.push(')');
      for value in rest {
        source.push_str(" and (");
        source.push_str(value);
        source.push(')');
      }
    }
    source.push(')');
  }
  if let Some(media) = media {
    source.push(' ');
    source.push_str(&media);
  }
  source.push(';');
  source
}

/// Merges a nested layer into its outer layer.
///
/// Anonymous layers have no addressable name, so they fall back to the closest
/// representable form: an anonymous outer layer stays anonymous, while an
/// anonymous inner layer keeps the named outer layer.
fn merge_css_layers(outer: CssLayer, inner: CssLayer) -> CssLayer {
  match (outer, inner) {
    (CssLayer::Named(outer), CssLayer::Named(inner)) => {
      CssLayer::Named(format!("{outer}.{inner}").into())
    }
    (CssLayer::Anonymous, _) => CssLayer::Anonymous,
    (CssLayer::Named(outer), CssLayer::Anonymous) => CssLayer::Named(outer),
  }
}

/// Intersects two media query lists according to CSS Media Queries Level 4
/// syntax, keeping at most one `[not | only]? <media-type>` at the start of
/// each query.
fn merge_media_query_lists(outer: &str, inner: &str) -> String {
  let outer = split_media_query_list(outer);
  let inner = split_media_query_list(inner);
  if outer.is_empty() {
    return inner.join(", ");
  }
  if inner.is_empty() {
    return outer.join(", ");
  }

  let outer_parsed: Vec<_> = outer.into_iter().map(parse_media_query).collect();
  let inner_parsed: Vec<_> = inner.into_iter().map(parse_media_query).collect();

  let mut merged = Vec::with_capacity(outer_parsed.len() * inner_parsed.len());
  for outer_query in &outer_parsed {
    for inner_query in &inner_parsed {
      for intersected in intersect_parsed_queries(outer_query, inner_query) {
        if let Some(formatted) = format_media_query(&intersected)
          && !merged.contains(&formatted)
        {
          merged.push(formatted);
        }
      }
    }
  }

  if merged.is_empty() {
    "not all".to_string()
  } else {
    merged.join(", ")
  }
}

fn consume_css_escape(s: &str, backslash_idx: usize) -> Option<(char, usize)> {
  let bytes = s.as_bytes();
  if bytes.get(backslash_idx) != Some(&b'\\') {
    return None;
  }
  let after_bs = backslash_idx + 1;
  let Some(ch) = s[after_bs..].chars().next() else {
    return Some((char::REPLACEMENT_CHARACTER, after_bs));
  };
  if matches!(ch, '\n' | '\r' | '\x0c') {
    return None;
  }
  if ch.is_ascii_hexdigit() {
    let mut pos = after_bs;
    let mut value = 0u32;
    let mut digits = 0;
    while digits < 6
      && let Some(&byte) = bytes.get(pos)
      && byte.is_ascii_hexdigit()
    {
      value = value * 16
        + match byte {
          b'0'..=b'9' => u32::from(byte - b'0'),
          b'a'..=b'f' => u32::from(byte - b'a' + 10),
          b'A'..=b'F' => u32::from(byte - b'A' + 10),
          _ => 0,
        };
      pos += 1;
      digits += 1;
    }
    if let Some(&ws) = bytes.get(pos)
      && matches!(ws, b' ' | b'\t' | b'\n' | b'\r' | 0x0C)
    {
      pos += 1;
      if ws == b'\r' && bytes.get(pos) == Some(&b'\n') {
        pos += 1;
      }
    }
    let decoded = if value == 0 {
      char::REPLACEMENT_CHARACTER
    } else {
      char::from_u32(value).unwrap_or(char::REPLACEMENT_CHARACTER)
    };
    return Some((decoded, pos));
  }
  Some((ch, after_bs + ch.len_utf8()))
}

fn split_media_query_list(media: &str) -> Vec<&str> {
  let mut queries = Vec::new();
  let mut depth = 0usize;
  let mut quote: Option<char> = None;
  let mut escaped = false;
  let mut start = 0;
  let mut i = 0;
  let bytes = media.as_bytes();

  while i < media.len() {
    if let Some(q) = quote {
      let Some(ch) = media[i..].chars().next() else {
        break;
      };
      if escaped {
        escaped = false;
      } else if ch == '\\' {
        escaped = true;
      } else if ch == q {
        quote = None;
      }
      i += ch.len_utf8();
      continue;
    }

    if bytes[i..].starts_with(b"/*") {
      if let Some(rel) = media[i + 2..].find("*/") {
        i += 2 + rel + 2;
        continue;
      }
      break;
    }

    let Some(ch) = media[i..].chars().next() else {
      break;
    };
    if ch == '\\'
      && let Some((_, next)) = consume_css_escape(media, i)
    {
      i = next;
      continue;
    }
    match ch {
      '"' | '\'' => quote = Some(ch),
      '(' | '[' => depth += 1,
      ')' | ']' => depth = depth.saturating_sub(1),
      ',' if depth == 0 => {
        let query = media[start..i].trim();
        if !query.is_empty() {
          queries.push(query);
        }
        start = i + 1;
      }
      _ => {}
    }
    i += ch.len_utf8();
  }

  let query = media[start..].trim();
  if !query.is_empty() {
    queries.push(query);
  }
  queries
}

fn skip_ws_and_comments(mut s: &str) -> &str {
  loop {
    s = s.trim_start_matches(|ch: char| ch.is_ascii_whitespace());
    if let Some(after_open) = s.strip_prefix("/*") {
      if let Some(close_idx) = after_open.find("*/") {
        s = &after_open[close_idx + 2..];
        continue;
      }
      return "";
    }
    return s;
  }
}

fn is_ident_start(ch: char) -> bool {
  ch.is_ascii_alphabetic() || ch == '_' || !ch.is_ascii()
}

fn is_ident_continue(ch: char) -> bool {
  ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || !ch.is_ascii()
}

fn plain_ident_prefix_len(s: &str) -> Option<usize> {
  let mut chars = s.char_indices();
  let (_, first) = chars.next()?;
  let mut end = if is_ident_start(first) {
    first.len_utf8()
  } else if first == '-' {
    let (_, second) = chars.next()?;
    if is_ident_start(second) || second == '-' {
      first.len_utf8() + second.len_utf8()
    } else {
      return None;
    }
  } else {
    return None;
  };
  for (idx, ch) in chars {
    if is_ident_continue(ch) {
      end = idx + ch.len_utf8();
    } else {
      break;
    }
  }
  Some(end)
}

fn is_plain_ident(s: &str) -> bool {
  !s.is_empty() && plain_ident_prefix_len(s) == Some(s.len())
}

fn consume_ident(s: &str) -> Option<(ParsedIdent<'_>, usize)> {
  let first = s.chars().next()?;
  let mut decoded: Option<String> = None;
  let mut i = if is_ident_start(first) {
    first.len_utf8()
  } else if first == '\\' {
    let (ch, next) = consume_css_escape(s, 0)?;
    let mut buf = String::with_capacity(s.len().min(32));
    buf.push(ch);
    decoded = Some(buf);
    next
  } else if first == '-' {
    let second = s[1..].chars().next()?;
    if is_ident_start(second) || second == '-' {
      1 + second.len_utf8()
    } else if second == '\\' {
      let (ch, next) = consume_css_escape(s, 1)?;
      let mut buf = String::with_capacity(s.len().min(32));
      buf.push('-');
      buf.push(ch);
      decoded = Some(buf);
      next
    } else {
      return None;
    }
  } else {
    return None;
  };

  while i < s.len() {
    let Some(ch) = s[i..].chars().next() else {
      break;
    };
    if is_ident_continue(ch) {
      if let Some(buf) = &mut decoded {
        buf.push(ch);
      }
      i += ch.len_utf8();
    } else if ch == '\\' {
      let Some((escaped_ch, next)) = consume_css_escape(s, i) else {
        break;
      };
      let buf = decoded.get_or_insert_with(|| s[..i].to_string());
      buf.push(escaped_ch);
      i = next;
    } else {
      break;
    }
  }

  let raw = &s[..i];
  let decoded = match decoded {
    Some(buf) => Cow::Owned(buf),
    None => Cow::Borrowed(raw),
  };
  Some((ParsedIdent { decoded, raw }, i))
}

fn take_leading_ident(s: &str) -> Option<(ParsedIdent<'_>, &str)> {
  let (ident, end) = consume_ident(s)?;
  let rest = &s[end..];
  if rest.is_empty()
    || rest.starts_with(|ch: char| ch.is_ascii_whitespace())
    || rest.starts_with("/*")
  {
    Some((ident, rest))
  } else {
    None
  }
}

fn starts_with_media_in_parens(s: &str) -> bool {
  s.starts_with('(') || consume_ident(s).is_some_and(|(_, len)| s[len..].starts_with('('))
}

fn consume_media_in_parens(s: &str) -> Option<(&str, &str)> {
  let open_idx = if s.starts_with('(') {
    0
  } else if let Some((_, len)) = consume_ident(s)
    && s[len..].starts_with('(')
  {
    len
  } else {
    return None;
  };

  let mut depth = 0usize;
  let mut quote: Option<char> = None;
  let mut escaped = false;
  let bytes = s.as_bytes();
  let mut i = open_idx;

  while i < s.len() {
    if let Some(q) = quote {
      let ch = s[i..].chars().next()?;
      if escaped {
        escaped = false;
      } else if ch == '\\' {
        escaped = true;
      } else if ch == q {
        quote = None;
      }
      i += ch.len_utf8();
      continue;
    }

    if bytes[i..].starts_with(b"/*") {
      if let Some(rel) = s[i + 2..].find("*/") {
        i += 2 + rel + 2;
        continue;
      }
      return None;
    }

    let ch = s[i..].chars().next()?;
    if ch == '\\'
      && let Some((_, next)) = consume_css_escape(s, i)
    {
      i = next;
      continue;
    }
    match ch {
      '"' | '\'' => quote = Some(ch),
      '(' => depth += 1,
      ')' => {
        depth -= 1;
        if depth == 0 {
          let end = i + 1;
          return Some((&s[..end], &s[end..]));
        }
      }
      _ => {}
    }
    i += ch.len_utf8();
  }

  None
}

fn is_valid_media_condition(s: &str, allow_or: bool) -> bool {
  let rest = skip_ws_and_comments(s);
  if let Some((ident, after)) = take_leading_ident(rest)
    && ident.decoded.eq_ignore_ascii_case("not")
  {
    let after = skip_ws_and_comments(after);
    let Some((_, tail)) = consume_media_in_parens(after) else {
      return false;
    };
    return skip_ws_and_comments(tail).is_empty();
  }

  let Some((_, mut tail)) = consume_media_in_parens(rest) else {
    return false;
  };
  let mut is_and_chain: Option<bool> = None;
  loop {
    tail = skip_ws_and_comments(tail);
    if tail.is_empty() {
      return true;
    }
    let Some((kw, after_kw)) = take_leading_ident(tail) else {
      return false;
    };
    if kw.decoded.eq_ignore_ascii_case("and") {
      if is_and_chain == Some(false) {
        return false;
      }
      is_and_chain = Some(true);
    } else if allow_or && kw.decoded.eq_ignore_ascii_case("or") {
      if is_and_chain == Some(true) {
        return false;
      }
      is_and_chain = Some(false);
    } else {
      return false;
    }
    let after_kw = skip_ws_and_comments(after_kw);
    let Some((_, next_tail)) = consume_media_in_parens(after_kw) else {
      return false;
    };
    tail = next_tail;
  }
}

fn is_valid_media_type(ident: &str) -> bool {
  !ident.eq_ignore_ascii_case("only")
    && !ident.eq_ignore_ascii_case("not")
    && !ident.eq_ignore_ascii_case("and")
    && !ident.eq_ignore_ascii_case("or")
    && !ident.eq_ignore_ascii_case("layer")
}

fn parse_media_query_tail(s: &str) -> Option<Option<String>> {
  let rest = skip_ws_and_comments(s);
  if rest.is_empty() {
    return Some(None);
  }
  let (kw, after_kw) = take_leading_ident(rest)?;
  if !kw.decoded.eq_ignore_ascii_case("and") {
    return None;
  }
  let cond = skip_ws_and_comments(after_kw);
  if cond.is_empty() || !is_valid_media_condition(cond, false) {
    return None;
  }
  Some(Some(cond.trim().to_string()))
}

fn parse_media_query(raw: &str) -> ParsedMediaQuery {
  let s = skip_ws_and_comments(raw);
  if s.is_empty() {
    return ParsedMediaQuery::Never;
  }

  let Some((first, after_first)) = take_leading_ident(s) else {
    return if is_valid_media_condition(s, true) {
      ParsedMediaQuery::Positive {
        only: false,
        media_type: None,
        condition: Some(s.trim().to_string()),
      }
    } else {
      ParsedMediaQuery::Never
    };
  };

  if first.decoded.eq_ignore_ascii_case("not") {
    let rest = skip_ws_and_comments(after_first);
    if starts_with_media_in_parens(rest) {
      return if is_valid_media_condition(s, true) {
        ParsedMediaQuery::Positive {
          only: false,
          media_type: None,
          condition: Some(s.trim().to_string()),
        }
      } else {
        ParsedMediaQuery::Never
      };
    }

    let Some((mt, after_mt)) = take_leading_ident(rest) else {
      return ParsedMediaQuery::Never;
    };
    if !is_valid_media_type(&mt.decoded) {
      return ParsedMediaQuery::Never;
    }
    let Some(cond) = parse_media_query_tail(after_mt) else {
      return ParsedMediaQuery::Never;
    };
    let media_type = MediaType::from_ident(&mt);
    return if media_type.eq_ignore_ascii_case("all") {
      match cond {
        None => ParsedMediaQuery::Never,
        Some(c) => ParsedMediaQuery::Positive {
          only: false,
          media_type: None,
          condition: Some(negate_condition(&c)),
        },
      }
    } else {
      ParsedMediaQuery::Negated {
        media_type,
        condition: cond,
      }
    };
  }

  if first.decoded.eq_ignore_ascii_case("only") {
    let rest = skip_ws_and_comments(after_first);
    let Some((mt, after_mt)) = take_leading_ident(rest) else {
      return ParsedMediaQuery::Never;
    };
    if !is_valid_media_type(&mt.decoded) {
      return ParsedMediaQuery::Never;
    }
    let Some(cond) = parse_media_query_tail(after_mt) else {
      return ParsedMediaQuery::Never;
    };
    return ParsedMediaQuery::Positive {
      only: true,
      media_type: Some(MediaType::from_ident(&mt)),
      condition: cond,
    };
  }

  if is_valid_media_type(&first.decoded) {
    let Some(cond) = parse_media_query_tail(after_first) else {
      return ParsedMediaQuery::Never;
    };
    return ParsedMediaQuery::Positive {
      only: false,
      media_type: Some(MediaType::from_ident(&first)),
      condition: cond,
    };
  }

  ParsedMediaQuery::Never
}

fn is_single_media_in_parens(s: &str) -> bool {
  let trimmed = skip_ws_and_comments(s);
  consume_media_in_parens(trimmed).is_some_and(|(_, tail)| skip_ws_and_comments(tail).is_empty())
}

fn needs_parens_in_and(s: &str) -> bool {
  let trimmed = skip_ws_and_comments(s);
  if is_single_media_in_parens(trimmed) {
    return false;
  }
  if let Some((kw, _)) = take_leading_ident(trimmed)
    && kw.decoded.eq_ignore_ascii_case("not")
  {
    return true;
  }
  let Some((_, mut tail)) = consume_media_in_parens(trimmed) else {
    return true;
  };
  loop {
    tail = skip_ws_and_comments(tail);
    if tail.is_empty() {
      return false;
    }
    let Some((kw, after_kw)) = take_leading_ident(tail) else {
      return true;
    };
    if !kw.decoded.eq_ignore_ascii_case("and") {
      return true;
    }
    let Some((_, next_tail)) = consume_media_in_parens(skip_ws_and_comments(after_kw)) else {
      return true;
    };
    tail = next_tail;
  }
}

fn wrap_for_and(condition: &str) -> Cow<'_, str> {
  if needs_parens_in_and(condition) {
    Cow::Owned(format!("({condition})"))
  } else {
    Cow::Borrowed(condition)
  }
}

fn negate_condition(condition: &str) -> String {
  let trimmed = condition.trim();
  if is_single_media_in_parens(trimmed) {
    format!("not {trimmed}")
  } else {
    format!("not ({trimmed})")
  }
}

fn combine_conditions(a: Option<&str>, b: Option<&str>) -> Option<String> {
  match (a, b) {
    (None, None) => None,
    (Some(c), None) | (None, Some(c)) => Some(c.to_string()),
    (Some(a), Some(b)) if a == b => Some(a.to_string()),
    (Some(a), Some(b)) => Some(format!("{} and {}", wrap_for_and(a), wrap_for_and(b))),
  }
}

fn is_unconstrained_all(query: &ParsedMediaQuery) -> bool {
  matches!(
    query,
    ParsedMediaQuery::Positive {
      only: false,
      media_type: None,
      condition: None,
    }
  ) || matches!(
    query,
    ParsedMediaQuery::Positive {
      only: false,
      media_type: Some(mt),
      condition: None,
    } if mt.eq_ignore_ascii_case("all")
  )
}

fn expand_negated_query(media_type: &MediaType, condition: Option<&str>) -> Vec<ParsedMediaQuery> {
  let mut expanded = Vec::new();
  for known in ["screen", "print", "speech"] {
    if !media_type.eq_ignore_ascii_case(known) {
      expanded.push(ParsedMediaQuery::Positive {
        only: false,
        media_type: Some(MediaType::from_plain(known)),
        condition: None,
      });
    }
  }
  if let Some(cond) = condition {
    expanded.push(ParsedMediaQuery::Positive {
      only: false,
      media_type: Some(media_type.clone()),
      condition: Some(negate_condition(cond)),
    });
  }
  expanded
}

fn intersect_parsed_queries(a: &ParsedMediaQuery, b: &ParsedMediaQuery) -> Vec<ParsedMediaQuery> {
  if matches!(a, ParsedMediaQuery::Never) || matches!(b, ParsedMediaQuery::Never) {
    return Vec::new();
  }
  if is_unconstrained_all(a) {
    return vec![b.clone()];
  }
  if is_unconstrained_all(b) {
    return vec![a.clone()];
  }

  match (a, b) {
    (
      ParsedMediaQuery::Positive {
        only: o1,
        media_type: t1,
        condition: c1,
      },
      ParsedMediaQuery::Positive {
        only: o2,
        media_type: t2,
        condition: c2,
      },
    ) => {
      let merged_type = match (t1.as_ref(), t2.as_ref()) {
        (None, Some(t)) | (Some(t), None) => Some(t.clone()),
        (None, None) => None,
        (Some(a), Some(b)) if a.eq_ignore_ascii_case("all") => Some(b.clone()),
        (Some(a), Some(b)) if b.eq_ignore_ascii_case("all") => Some(a.clone()),
        (Some(a), Some(b)) if a.eq_ignore_ascii_case(&b.decoded) => Some(a.clone()),
        (Some(_), Some(_)) => return Vec::new(),
      };
      let merged_cond = combine_conditions(c1.as_deref(), c2.as_deref());
      let only = (*o1 || *o2) && merged_type.is_some();
      vec![ParsedMediaQuery::Positive {
        only,
        media_type: merged_type,
        condition: merged_cond,
      }]
    }
    (
      ParsedMediaQuery::Negated {
        media_type: nt,
        condition: nc,
      },
      ParsedMediaQuery::Positive {
        only,
        media_type: Some(pt),
        condition: pc,
      },
    )
    | (
      ParsedMediaQuery::Positive {
        only,
        media_type: Some(pt),
        condition: pc,
      },
      ParsedMediaQuery::Negated {
        media_type: nt,
        condition: nc,
      },
    ) if !pt.eq_ignore_ascii_case("all") => {
      if !nt.eq_ignore_ascii_case(&pt.decoded) {
        vec![ParsedMediaQuery::Positive {
          only: *only,
          media_type: Some(pt.clone()),
          condition: pc.clone(),
        }]
      } else if let Some(nc) = nc {
        vec![ParsedMediaQuery::Positive {
          only: *only,
          media_type: Some(pt.clone()),
          condition: combine_conditions(Some(&negate_condition(nc)), pc.as_deref()),
        }]
      } else {
        Vec::new()
      }
    }
    (
      ParsedMediaQuery::Negated {
        media_type: t1,
        condition: None,
      },
      ParsedMediaQuery::Negated {
        media_type: t2,
        condition: None,
      },
    ) if t1.eq_ignore_ascii_case(&t2.decoded) => {
      vec![ParsedMediaQuery::Negated {
        media_type: t1.clone(),
        condition: None,
      }]
    }
    (
      ParsedMediaQuery::Negated {
        media_type: nt,
        condition: nc,
      },
      other,
    ) => expand_negated_query(nt, nc.as_deref())
      .into_iter()
      .flat_map(|expanded| intersect_parsed_queries(&expanded, other))
      .collect(),
    (
      other,
      ParsedMediaQuery::Negated {
        media_type: nt,
        condition: nc,
      },
    ) => expand_negated_query(nt, nc.as_deref())
      .into_iter()
      .flat_map(|expanded| intersect_parsed_queries(other, &expanded))
      .collect(),
    _ => Vec::new(),
  }
}

fn format_media_query(query: &ParsedMediaQuery) -> Option<String> {
  match query {
    ParsedMediaQuery::Never => None,
    ParsedMediaQuery::Positive {
      only,
      media_type: Some(mt),
      condition: Some(cond),
    } => Some(format!(
      "{}{} and {}",
      if *only { "only " } else { "" },
      mt.output,
      wrap_for_and(cond)
    )),
    ParsedMediaQuery::Positive {
      only,
      media_type: Some(mt),
      condition: None,
    } => Some(format!("{}{}", if *only { "only " } else { "" }, mt.output)),
    ParsedMediaQuery::Positive {
      media_type: None,
      condition: Some(cond),
      ..
    } => Some(cond.clone()),
    ParsedMediaQuery::Positive {
      media_type: None,
      condition: None,
      ..
    } => Some("all".to_string()),
    ParsedMediaQuery::Negated {
      media_type,
      condition: Some(cond),
    } => Some(format!(
      "not {} and {}",
      media_type.output,
      wrap_for_and(cond)
    )),
    ParsedMediaQuery::Negated {
      media_type,
      condition: None,
    } => Some(format!("not {}", media_type.output)),
  }
}

impl ExternalModule {
  fn create_identifier(
    request: &ExternalRequest,
    external_type: &str,
    dependency_meta: &DependencyMeta,
    interop: Option<ExternalInterop>,
  ) -> Identifier {
    let resolved_type = resolve_external_type(external_type, dependency_meta);
    let request_str = simd_json::to_string(request).expect("invalid json to_string");
    let attrs_str = dependency_meta
      .attributes
      .as_ref()
      .map_or(String::new(), |attrs| {
        format!(
          " {}",
          simd_json::to_string(attrs).expect("invalid json to_string")
        )
      });
    let phase_str = if dependency_meta.phase == ImportPhase::Evaluation {
      String::new()
    } else {
      format!(" phase={}", dependency_meta.phase.as_str())
    };
    let css_import_str =
      css_module_render_conditions_identifier(dependency_meta.css_import_conditions.iter())
        .map_or(String::new(), |conditions| {
          format!(" css-import-conditions={}", json_stringify_str(&conditions))
        });
    let interop_str = interop.map_or(String::new(), |interop| {
      format!("|interop={}", interop.as_str())
    });
    Identifier::from(format!(
      "external {resolved_type} {request_str}{attrs_str}{phase_str}{css_import_str}{interop_str}"
    ))
  }

  pub fn new(
    request: ExternalRequest,
    external_type: ExternalType,
    user_request: String,
    dependency_meta: DependencyMeta,
    place_in_initial: bool,
    interop: Option<ExternalInterop>,
  ) -> Self {
    Self {
      dependencies_block: Default::default(),
      id: Self::create_identifier(&request, &external_type, &dependency_meta, interop),
      request,
      external_type,
      user_request,
      factory_meta: Default::default(),
      build_info: BuildInfo {
        top_level_declarations: Some(Default::default()),
        strict: true,
        ..Default::default()
      }
      .into(),
      build_meta: Default::default(),
      source_map_kind: SourceMapKind::empty(),
      dependency_meta,
      place_in_initial,
      interop,
    }
  }

  pub fn user_request(&self) -> &str {
    &self.user_request
  }

  pub fn set_id(&mut self, id: Identifier) {
    self.id = id;
  }

  pub fn get_external_type(&self) -> &ExternalType {
    &self.external_type
  }

  pub fn get_import_phase(&self) -> ImportPhase {
    self.dependency_meta.phase
  }

  pub fn resolve_external_type(&self) -> &str {
    resolve_external_type(self.external_type.as_str(), &self.dependency_meta)
  }

  /// Renders the CSS text of a `css-import` external module.
  ///
  /// These modules have no source of their own, their stylesheet is generated
  /// from the external request and the `@import` conditions.
  pub fn css_import_source(&self) -> Option<String> {
    if self.resolve_external_type() != "css-import" {
      return None;
    }
    let request = self.try_get_request()?;
    Some(render_css_import_source(
      request.primary(),
      &self.dependency_meta.css_import_conditions,
    ))
  }

  pub async fn chunk_condition_with_hooks(
    &self,
    chunk_ukey: &ChunkUkey,
    compilation: &Compilation,
  ) -> Result<Option<bool>> {
    if let Some(condition) = compilation
      .plugin_driver
      .compilation_hooks
      .external_module_chunk_condition
      .call(chunk_ukey, compilation)
      .await?
    {
      return Ok(Some(condition));
    }

    Ok(<Self as Module>::chunk_condition(
      self,
      chunk_ukey,
      compilation,
    ))
  }

  pub fn set_external_type(&mut self, new_type: ExternalType) {
    if let ExternalRequest::Map(map) = &mut self.request
      && !map.contains_key(&new_type)
      && let Some(request) = map.get(&self.external_type).cloned()
    {
      // Preserve the request selected by the previous type when a plugin
      // changes how the external is rendered. Object-form externals are keyed
      // by type, so changing only the type would make `get_request` panic.
      map.insert(new_type.clone(), request);
    }
    self.external_type = new_type;
    self.id = Self::create_identifier(
      &self.request,
      &self.external_type,
      &self.dependency_meta,
      self.interop,
    );
  }

  pub fn get_request(&self) -> &ExternalRequestValue {
    match &self.request {
      ExternalRequest::Single(request) => request,
      ExternalRequest::Map(map) => &map[&self.external_type],
    }
  }

  pub fn try_get_request(&self) -> Option<&ExternalRequestValue> {
    self.get_request_and_external_type().0
  }

  fn get_request_and_external_type(&self) -> (Option<&ExternalRequestValue>, &ExternalType) {
    match &self.request {
      ExternalRequest::Single(request) => (Some(request), &self.external_type),
      ExternalRequest::Map(map) => (map.get(&self.external_type), &self.external_type),
    }
  }

  fn get_source(
    &self,
    compilation: &Compilation,
    request: Option<&ExternalRequestValue>,
    external_type: &ExternalType,
    runtime: Option<&RuntimeSpec>,
    concatenation_scope: Option<&mut ConcatenationScope>,
    runtime_template: &mut ModuleCodeTemplate,
  ) -> Result<(BoxSource, ChunkInitFragments)> {
    let mut chunk_init_fragments: ChunkInitFragments = Default::default();
    let supports_const = compilation.options.output.environment.supports_const();
    let resolved_external_type = self.resolve_external_type();
    if let Some(require_kind) =
      CommonJsExternalRequireKind::from_external_type(resolved_external_type)
    {
      // For a missing object-form request, only ESM node-commonjs uses the
      // undefined value; other CommonJS types request the name "undefined".
      let fallback = (require_kind != CommonJsExternalRequireKind::NodeCommonJs
        || !compilation.options.output.module)
        .then_some("undefined");
      let require_expression = require_kind.render_expression(
        request.map(ExternalRequestValue::primary).or(fallback),
        request
          .and_then(ExternalRequestValue::rest)
          .unwrap_or_default(),
        compilation,
        &mut chunk_init_fragments,
      );
      let source = format!(
        "{} = {require_expression};",
        get_namespace_object_export(concatenation_scope, supports_const, runtime_template)
      );
      return Ok((RawStringSource::from(source).boxed(), chunk_init_fragments));
    }
    let module_graph = compilation.get_module_graph();
    let module_graph_cache = &compilation.module_graph_cache_artifact;

    let source = match resolved_external_type {
      "this" => format!(
        "{} = (function() {{ return {}; }}());",
        get_namespace_object_export(concatenation_scope, supports_const, runtime_template),
        get_source_for_global_variable_external(request, external_type),
      ),
      "window" | "self" => format!(
        "{} = {};",
        get_namespace_object_export(concatenation_scope, supports_const, runtime_template),
        get_source_for_global_variable_external(request, external_type)
      ),
      "global" => format!(
        "{} = {};",
        get_namespace_object_export(concatenation_scope, supports_const, runtime_template),
        get_source_for_global_variable_external(request, &compilation.options.output.global_object)
      ),
      "amd" | "amd-require" | "umd" | "umd2" | "system" | "jsonp" => {
        let id = ChunkGraph::get_module_id(&compilation.module_ids_artifact, self.identifier())
          .map(|s| s.as_str())
          .expect("should have module id");
        let external_variable = format!("__rspack_external_{}", to_identifier(id));
        let side_effects_state_artifact = &compilation
          .build_module_graph_artifact
          .side_effects_state_artifact;
        let check_external_variable = if module_graph.is_optional(
          &self.id,
          module_graph_cache,
          side_effects_state_artifact,
          &compilation.exports_info_artifact,
        ) {
          format!(
            "if(typeof {} === 'undefined') {{ {} }}\n",
            external_variable,
            runtime_template.throw_missing_module_error_block(&self.user_request)
          )
        } else {
          String::new()
        };
        format!(
          "{}{} = {};",
          check_external_variable,
          get_namespace_object_export(concatenation_scope, supports_const, runtime_template),
          external_variable
        )
      }
      "import" => format!(
        "{} = {};",
        get_namespace_object_export(concatenation_scope, supports_const, runtime_template),
        get_source_for_import(
          request,
          compilation,
          &self.dependency_meta.attributes,
          self.dependency_meta.phase,
        )
      ),
      "var" | "promise" | "const" | "let" | "assign" => {
        let external_variable = if let Some(request) = request {
          get_request_string(request)
        } else {
          "undefined".to_string()
        };
        let side_effects_state_artifact = &compilation
          .build_module_graph_artifact
          .side_effects_state_artifact;
        let check_external_variable = if module_graph.is_optional(
          &self.id,
          module_graph_cache,
          side_effects_state_artifact,
          &compilation.exports_info_artifact,
        ) && let Some(request) = request
        {
          format!(
            "if(typeof {} === 'undefined') {{ {} }}\n",
            external_variable,
            runtime_template.throw_missing_module_error_block(&get_request_string(request))
          )
        } else {
          String::new()
        };
        format!(
          "{}{} = {};",
          check_external_variable,
          get_namespace_object_export(concatenation_scope, supports_const, runtime_template),
          external_variable
        )
      }
      "module" => {
        if compilation.options.output.module
          && let Some(request) = request
        {
          let id: Cow<'_, str> = if to_identifier(&request.primary) != request.primary
            || self.dependency_meta.attributes.is_some()
          {
            let mut hasher = RspackHasher::from(&compilation.options.output);
            use rspack_hash::RspackHash as _;
            request.primary.hash(&mut hasher);
            if let Some(attributes) = &self.dependency_meta.attributes {
              simd_json::to_string(attributes)
                .expect("json stringify failed")
                .hash(&mut hasher);
            }
            let hash_suffix = hasher.digest(&compilation.options.output.hash_digest);
            Cow::Owned(format!(
              "{}_{}",
              to_identifier(&request.primary),
              hash_suffix.rendered(8)
            ))
          } else {
            to_identifier(&request.primary)
          };
          if let Some(concatenation_scope) = concatenation_scope {
            let exports_info = compilation
              .exports_info_artifact
              .get_exports_info_data(&self.identifier());
            let used_exports = exports_info.get_used_exports(runtime);
            let namespace_used_by_named_exports = matches!(
              &used_exports,
              UsedExports::UsedNames(atoms)
                if atoms
                  .iter()
                  .any(|atom| {
                    exports_info
                      .get_read_only_export_info(atom)
                      .ns_access()
                  })
            );
            let attributes = self.dependency_meta.attributes.as_ref().map(|meta| {
              format!(
                " with {}",
                simd_json::to_string(meta).expect("json stringify failed"),
              )
            });

            #[derive(Clone, Copy)]
            struct ExternalImportOptimize(pub bool);

            let safe_to_optimize =
              if let Some(optimize) = concatenation_scope.data.get::<ExternalImportOptimize>() {
                optimize.0
              } else {
                let chunk = compilation
                  .build_chunk_graph_artifact
                  .chunk_graph
                  .get_module_chunks(concatenation_scope.concat_module_id);

                let safe_to_optimize = if chunk.is_empty() {
                  false
                } else {
                  let chunk = chunk
                    .iter()
                    .next()
                    .expect("concate module have only 1 chunk");
                  let module_graph = compilation.get_module_graph();
                  let mut safe_to_optimize = true;
                  'outer: for m in compilation
                    .build_chunk_graph_artifact
                    .chunk_graph
                    .get_chunk_modules(chunk, module_graph)
                  {
                    if m.identifier() == concatenation_scope.concat_module_id {
                      continue;
                    }

                    if let Some(m) = m.as_concatenated_module() {
                      for m in m.get_modules() {
                        if let Some(external_module) = module_graph
                          .module_by_identifier(&m.id)
                          .expect("should have module")
                          .as_external_module()
                          && external_module.resolve_external_type() == "module"
                        {
                          safe_to_optimize = false;
                          break 'outer;
                        }
                      }
                    }
                  }
                  safe_to_optimize
                };

                concatenation_scope
                  .data
                  .insert(ExternalImportOptimize(safe_to_optimize));
                safe_to_optimize
              };

            let force_namespace = request.has_rest() || self.dependency_meta.phase.is_source();

            match used_exports {
              UsedExports::UsedNamespace(true) | UsedExports::Unknown => {
                let external_module_id = format!("__rspack_external_{id}");
                let namespace_export_with_name = format!(
                  "{}{}{}",
                  NAMESPACE_OBJECT_EXPORT,
                  external_module_id,
                  property_access(request.iter(), 1)
                );

                concatenation_scope.register_namespace_import(
                  request.primary().to_string(),
                  attributes,
                  format!("__rspack_external_{id}").into(),
                );
                concatenation_scope.register_namespace_export(&namespace_export_with_name);
                String::new()
              }
              UsedExports::UsedNamespace(false) => {
                concatenation_scope.register_import(
                  request.primary().to_string(),
                  attributes,
                  None,
                );
                String::new()
              }
              UsedExports::UsedNames(atoms) => {
                if !safe_to_optimize || namespace_used_by_named_exports || force_namespace {
                  if collect_module_external_remapping(
                    &compilation.exports_info_artifact,
                    exports_info,
                    runtime,
                  )
                  .is_some()
                  {
                    let (init, expression, module_external_fragments) =
                      get_source_for_module_external(
                        request,
                        id.as_ref(),
                        &self.dependency_meta,
                        &compilation.exports_info_artifact,
                        exports_info,
                        runtime,
                        runtime_template,
                      );
                    chunk_init_fragments.extend(module_external_fragments);
                    let external_module_id = format!("__rspack_module_external_namespace_{id}");
                    let namespace_export_with_name =
                      format!("{NAMESPACE_OBJECT_EXPORT}{external_module_id}");
                    concatenation_scope.register_namespace_export(&namespace_export_with_name);
                    format!(
                      "{}\nvar {external_module_id} = {expression};",
                      init.expect("remapped module external should render init fragments")
                    )
                  } else {
                    let external_module_id = format!("__rspack_external_{id}");
                    let fragment = NormalInitFragment::new(
                      module_external_import_statement(
                        request,
                        id.as_ref(),
                        &self.dependency_meta.attributes,
                        self.dependency_meta.phase,
                      ),
                      InitFragmentStage::StageESMImports,
                      module_graph
                        .get_pre_order_index(&self.identifier())
                        .map_or(0, |num| num as i32),
                      InitFragmentKey::ModuleExternal(module_external_fragment_key(
                        request.primary(),
                        &self.dependency_meta.attributes,
                        self.dependency_meta.phase,
                      )),
                      None,
                    )
                    .with_top_level_decl_symbols(vec![external_module_id.clone().into()]);
                    chunk_init_fragments.push(fragment.boxed());
                    let namespace_export_with_name = format!(
                      "{}{}{}",
                      NAMESPACE_OBJECT_EXPORT,
                      external_module_id,
                      property_access(request.iter(), 1)
                    );
                    concatenation_scope.register_namespace_export(&namespace_export_with_name);
                    String::new()
                  }
                } else {
                  concatenation_scope.register_import(
                    request.primary().to_string(),
                    attributes.clone(),
                    None,
                  );
                  for atom in &atoms {
                    concatenation_scope.register_import(
                      request.primary().to_string(),
                      attributes.clone(),
                      Some(atom.clone()),
                    );
                    concatenation_scope.register_raw_export(atom.clone(), atom.to_string());
                  }
                  String::new()
                }
              }
            }
          } else {
            let exports_info = compilation
              .exports_info_artifact
              .get_exports_info_data(&self.identifier());
            let (init, expression, module_external_fragments) = get_source_for_module_external(
              request,
              id.as_ref(),
              &self.dependency_meta,
              &compilation.exports_info_artifact,
              exports_info,
              runtime,
              runtime_template,
            );
            chunk_init_fragments.extend(module_external_fragments);
            let export =
              get_namespace_object_export(concatenation_scope, supports_const, runtime_template);
            if let Some(init) = init {
              format!("{init}\n{export} = {expression};")
            } else {
              format!(
                r#"
{export} = {expression};
"#
              )
            }
          }
        } else {
          format!(
            "{} = {};",
            get_namespace_object_export(concatenation_scope, supports_const, runtime_template),
            get_source_for_import(
              request,
              compilation,
              &self.dependency_meta.attributes,
              self.dependency_meta.phase,
            )
          )
        }
      }
      "script" if request.is_some() => {
        let request = request.expect("request should be some");
        let url_and_global = extract_url_and_global(request.primary())?;
        format!(
          r#"
var __rspack_error = new Error();
{export} = new Promise(function(resolve, reject) {{
if(typeof {global} !== "undefined") return resolve();
{load_script}({url_str}, function(event) {{
  if(typeof {global} !== "undefined") return resolve();
  var errorType = event && (event.type === 'load' ? 'missing' : event.type);
  var realSrc = event && event.target && event.target.src;
  __rspack_error.message = 'Loading script failed.\n(' + errorType + ': ' + realSrc + ')';
  __rspack_error.name = 'ScriptExternalLoadError';
  __rspack_error.type = errorType;
  __rspack_error.request = realSrc;
  reject(__rspack_error);
}}, {global_str});
}}).then(function() {{ return {global}; }});
"#,
          export =
            get_namespace_object_export(concatenation_scope, supports_const, runtime_template),
          global = url_and_global.global,
          global_str = rspack_util::json_stringify_str(url_and_global.global),
          url_str = rspack_util::json_stringify_str(url_and_global.url),
          load_script = runtime_template.render_runtime_globals(&RuntimeGlobals::LOAD_SCRIPT)
        )
      }
      _ => {
        let external_variable = if let Some(request) = request {
          get_request_string(request)
        } else {
          "undefined".to_string()
        };
        let check_external_variable = if module_graph.is_optional(
          &self.id,
          module_graph_cache,
          &compilation
            .build_module_graph_artifact
            .side_effects_state_artifact,
          &compilation.exports_info_artifact,
        ) {
          format!(
            "if(typeof {} === 'undefined') {{ {} }}\n",
            external_variable,
            runtime_template.throw_missing_module_error_block(&external_variable)
          )
        } else {
          String::new()
        };
        format!(
          "{}{} = {};",
          check_external_variable,
          get_namespace_object_export(concatenation_scope, supports_const, runtime_template),
          external_variable,
        )
      }
    };
    Ok((RawStringSource::from(source).boxed(), chunk_init_fragments))
  }
}

impl Identifiable for ExternalModule {
  fn identifier(&self) -> Identifier {
    self.id
  }
}

impl DependenciesBlock for ExternalModule {
  fn dependencies_block(&self) -> &DependenciesBlockData {
    &self.dependencies_block
  }

  fn dependencies_block_mut(&mut self) -> &mut DependenciesBlockData {
    &mut self.dependencies_block
  }
}

#[cacheable_dyn]
#[async_trait::async_trait]
impl Module for ExternalModule {
  impl_module_meta_info!();

  fn get_concatenation_bailout_reason(
    &self,
    _mg: &ModuleGraph,
    _cg: &ChunkGraph,
  ) -> Option<Cow<'static, str>> {
    match self.external_type.as_ref() {
      "amd" | "umd" | "amd-require" | "umd2" | "system" | "jsonp" => {
        // return `${this.externalType} externals can't be concatenated`;
        Some(format!("{} externals can't be concatenated", self.external_type).into())
      }
      _ => None,
    }
  }

  fn module_type(&self) -> &ModuleType {
    &ModuleType::JsDynamic
  }

  fn source_types(&self, _module_graph: &ModuleGraph) -> &[SourceType] {
    match self.resolve_external_type() {
      "asset-url" => EXTERNAL_MODULE_ASSET_URL_SOURCE_TYPES,
      "css-import" => EXTERNAL_MODULE_CSS_SOURCE_TYPES,
      _ => EXTERNAL_MODULE_JS_SOURCE_TYPES,
    }
  }

  fn chunk_condition(&self, chunk_key: &ChunkUkey, compilation: &Compilation) -> Option<bool> {
    let resolved_external_type = self.resolve_external_type();
    let external_type = if self.external_type == "modern-module" {
      resolved_external_type
    } else {
      self.external_type.as_str()
    };

    match external_type {
      "css-import" | "module" | "import" | "module-import"
        if !self.place_in_initial && resolved_external_type != "import" =>
      {
        Some(true)
      }
      _ => Some(
        compilation
          .build_chunk_graph_artifact
          .chunk_graph
          .get_number_of_entry_modules(chunk_key)
          > 0,
      ),
    }
  }

  fn source(&self) -> Option<&BoxSource> {
    None
  }

  fn readable_identifier(&self, _context: &Context) -> Cow<'_, str> {
    Cow::Owned(format!(
      "external {}",
      simd_json::to_string(&self.request).expect("invalid json to_string")
    ))
  }

  fn size(&self, _source_type: Option<&SourceType>) -> f64 {
    // copied from webpack `ExternalModule`
    // roughly for url
    42.0
  }

  async fn need_build(&self, _context: &NeedBuildContext<'_>) -> Result<bool> {
    Ok(self.build_meta.frozen().is_none())
  }

  async fn build(
    mut self: Box<Self>,
    build_context: Arc<BuildContext>,
    _: Option<&Compilation>,
  ) -> Result<BoxModule> {
    self.build_info.get_mut().module = build_context.compiler_options.output.module;
    let resolved_external_type = self.resolve_external_type();
    let request = match &self.request {
      ExternalRequest::Single(request) => Some(request),
      ExternalRequest::Map(map) => map.get(&self.external_type),
    };
    let has_property_selection = match (resolved_external_type, &self.request) {
      ("umd" | "umd2", ExternalRequest::Map(map)) => ["root", "commonjs", "commonjs2", "amd"]
        .iter()
        .any(|external_type| {
          map
            .get(*external_type)
            .is_some_and(|request| request.has_rest())
        }),
      _ => request.is_some_and(|request| request.has_rest()),
    };
    let mut can_mangle = false;
    let mut exports_type = BuildMetaExportsType::Dynamic;

    #[allow(clippy::collapsible_match)]
    match resolved_external_type {
      "this" => self.build_info.get_mut().strict = false,
      "system" => {
        if !request.is_some_and(|r| r.has_rest()) {
          exports_type = BuildMetaExportsType::Namespace;
          can_mangle = true;
        }
      }
      "module" => {
        if self.build_info.get_mut().module {
          if !request.is_some_and(|r| r.has_rest()) {
            exports_type = BuildMetaExportsType::Namespace;
            can_mangle = true;
          }
        } else {
          self.build_meta.get_mut().set_has_top_level_await(true);
          if !request.is_some_and(|r| r.has_rest()) {
            exports_type = BuildMetaExportsType::Namespace;
            can_mangle = false;
          }
        }
      }
      "script" | "promise" => self.build_meta.get_mut().set_has_top_level_await(true),
      "import" => {
        self.build_meta.get_mut().set_has_top_level_await(true);
        if !request.is_some_and(|r| r.has_rest()) {
          exports_type = BuildMetaExportsType::Namespace;
          can_mangle = false;
        }
      }
      _ => {}
    }
    // Property-selected externals keep their existing dynamic export semantics.
    if let Some(interop) = self.interop
      && !has_property_selection
    {
      match interop {
        ExternalInterop::EsModule => exports_type = BuildMetaExportsType::Namespace,
        ExternalInterop::Default => {
          exports_type = BuildMetaExportsType::Default;
          self
            .build_meta
            .get_mut()
            .set_default_object(BuildMetaDefaultObject::Redirect);
        }
      }
      can_mangle = false;
    }
    self.build_meta.get_mut().set_exports_type(exports_type);
    Ok(BoxModule::new(self).with_dependencies(
      vec![DependencyRef::new(StaticExportsDependency::new(
        StaticExportsSpec::True,
        can_mangle,
      ))],
      Vec::new(),
    ))
  }

  // #[tracing::instrument("ExternalModule::code_generation", skip_all, fields(identifier = ?self.identifier()))]
  async fn code_generation(
    &self,
    code_generation_context: &mut ModuleCodeGenerationContext,
  ) -> Result<CodeGenerationResultBuilder> {
    let ModuleCodeGenerationContext {
      compilation,
      runtime,
      concatenation_scope,
      runtime_template,
    } = code_generation_context;

    let mut cgr = CodeGenerationResultBuilder::default();
    let (request, external_type) = self.get_request_and_external_type();
    match self.resolve_external_type() {
      "asset" if request.is_some() => {
        let request = request.expect("request should be some");
        cgr.add(
          SourceType::JavaScript,
          RawStringSource::from(format!(
            "{}.exports = {};",
            runtime_template.render_module_argument(ModuleArgument::Module),
            rspack_util::json_stringify_str(request.primary())
          ))
          .boxed(),
        );
        cgr
          .data_mut()
          .insert(CodeGenerationDataUrl::new(request.primary().to_string()));
      }
      "asset-url" if request.is_some() => {
        let request = request.expect("request should be some");
        cgr
          .data_mut()
          .insert(CodeGenerationDataUrl::new(request.primary().to_string()));
      }
      "css-import" => {
        if let Some(source) = self.css_import_source() {
          cgr.add(SourceType::Css, RawStringSource::from(source).boxed());
        }
      }
      _ => {
        let (source, chunk_init_fragments) = self.get_source(
          compilation,
          request,
          external_type,
          *runtime,
          concatenation_scope.as_mut(),
          runtime_template,
        )?;
        cgr.add(SourceType::JavaScript, source);
        if !chunk_init_fragments.is_empty() {
          cgr
            .data_mut()
            .insert(CodeGenerationDataChunkInitFragments::from(
              chunk_init_fragments,
            ));
        }
      }
    };
    Ok(cgr)
  }

  fn lib_ident(&self, _options: LibIdentOptions) -> Option<Cow<'_, str>> {
    Some(Cow::Borrowed(self.user_request.as_str()))
  }

  async fn get_runtime_hash(
    &self,
    compilation: &Compilation,
    runtime: Option<&RuntimeSpec>,
  ) -> Result<RspackHashDigest> {
    let mut hasher = RspackHasher::from(&compilation.options.output);
    use rspack_hash::RspackHash as _;
    self.id.as_str().hash(&mut hasher);
    let side_effects_state_artifact = &compilation
      .build_module_graph_artifact
      .side_effects_state_artifact;
    let is_optional = compilation.get_module_graph().is_optional(
      &self.id,
      &compilation.module_graph_cache_artifact,
      side_effects_state_artifact,
      &compilation.exports_info_artifact,
    );
    is_optional.hash(&mut hasher);
    module_update_hash(self, &mut hasher, compilation, runtime);
    Ok(hasher.digest(&compilation.options.output.hash_digest))
  }
}

#[inline]
pub async fn module_chunk_condition(
  module: &dyn Module,
  chunk_ukey: &ChunkUkey,
  compilation: &Compilation,
) -> Result<Option<bool>> {
  if let Some(external_module) = module.as_external_module() {
    external_module
      .chunk_condition_with_hooks(chunk_ukey, compilation)
      .await
  } else {
    Ok(module.chunk_condition(chunk_ukey, compilation))
  }
}

impl_empty_diagnosable_trait!(ExternalModule);
