use std::{borrow::Cow, cell::OnceCell, path::Path, sync::LazyLock};

use cow_utils::CowUtils;
use regex::Regex;
use rspack_core::{ChunkGraph, Compilation, OutputOptions, contextify, runtime_mode::RuntimeMode};
use rspack_error::Result;
use rspack_hash::{RspackHash, RspackHasher};
use rspack_paths::Utf8Path;
use rspack_util::identifier::split_at_query_mark;
use rustc_hash::FxHashMap as HashMap;
use sugar_path::SugarPath;

use crate::{ModuleFilenameTemplateFn, ModuleFilenameTemplateFnCtx, SourceReference};

fn get_before<'a>(s: &'a str, token: &str) -> &'a str {
  match s.rfind(token) {
    Some(idx) => &s[..idx],
    None => "",
  }
}

fn get_after<'a>(s: &'a str, token: &str) -> &'a str {
  s.find(token).map_or_default(|idx| &s[idx..])
}

fn get_hash(text: &str, output_options: &OutputOptions) -> String {
  let OutputOptions {
    hash_function,
    hash_salt,
    ..
  } = output_options;
  let mut hasher = RspackHasher::with_salt(hash_function, hash_salt);
  text.hash(&mut hasher);
  let mut buf = format!("{:x}", hasher.finish());
  buf.truncate(4);
  buf
}

pub struct ModuleFilenameHelpers;

static DEFAULT_RESOURCE_PATH_TEMPLATE_REGEXP: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(r"^(webpack://|rspack://)\[namespace\]/\[resourcePath\]$")
    .expect("failed to compile DEFAULT_RESOURCE_PATH_TEMPLATE_REGEXP regex")
});

fn default_resource_path_template_scheme(module_filename_template: &str) -> Option<&str> {
  DEFAULT_RESOURCE_PATH_TEMPLATE_REGEXP
    .captures(module_filename_template)
    .and_then(|captures| captures.get(1).map(|scheme| scheme.as_str()))
}

fn is_windows_path(path: &str) -> bool {
  let bytes = path.as_bytes();
  path.starts_with(r"\\")
    || (bytes.len() >= 3
      && bytes[0].is_ascii_alphabetic()
      && bytes[1] == b':'
      && matches!(bytes[2], b'/' | b'\\'))
}

fn has_url_scheme(resource: &str) -> bool {
  let Some((scheme, _)) = resource.split_once(':') else {
    return false;
  };
  let mut chars = scheme.chars();
  !is_windows_path(resource)
    && matches!(chars.next(), Some(ch) if ch.is_ascii_alphabetic())
    && chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '+' | '-' | '.'))
}

fn resource_path_end(resource: &str) -> usize {
  // The question mark in a DOS device prefix is part of the path.
  let prefix_len = if resource.starts_with(r"\\?\") || resource.starts_with(r"\\.\") {
    4
  } else {
    0
  };
  let mut escaped = false;
  for (offset, ch) in resource[prefix_len..].char_indices() {
    if escaped {
      escaped = false;
    } else if ch == '\u{200b}' {
      escaped = true;
    } else if matches!(ch, '?' | '#') {
      return prefix_len + offset;
    }
  }
  resource.len()
}

fn should_normalize_source_path(
  source_reference: &SourceReference,
  compilation: &Compilation,
) -> bool {
  match source_reference {
    SourceReference::Module(identifier) => compilation
      .get_module_graph()
      .module_by_identifier(identifier)
      .and_then(|module| module.as_normal_module())
      .and_then(|module| module.resource_resolved_data().path())
      .is_some(),
    // Composite identifiers are opaque, even when they contain filesystem paths.
    SourceReference::Source(source) => !source.contains(['|', '!']),
  }
}

fn push_normalized_resource(buf: &mut String, resource: &str) {
  if !resource.contains('\\')
    || (!cfg!(windows) && !is_windows_path(resource))
    || has_url_scheme(resource)
  {
    buf.push_str(resource);
    return;
  }

  let path_end = resource_path_end(resource);
  buf.push_str(&resource[..path_end].cow_replace('\\', "/"));
  buf.push_str(&resource[path_end..]);
}

// sources in a source map should be relative/URL-style (not absolute filesystem paths)
fn resolve_relative_resource_path(
  absolute_resource_path: &str,
  source_map_path: Option<&Utf8Path>,
  runtime_mode: RuntimeMode,
  normalize_path: bool,
  has_suffix: bool,
) -> Option<String> {
  if absolute_resource_path.starts_with("webpack/runtime")
    || absolute_resource_path.starts_with("rspack/runtime")
    || matches!(
      absolute_resource_path,
      "webpack/bootstrap" | "webpack/startup" | "rspack/bootstrap" | "rspack/startup"
    )
  {
    // Runtime modules, bootstrap, and startup are virtual.
    let scheme = match runtime_mode {
      RuntimeMode::Webpack => "webpack",
      RuntimeMode::Rspack => "rspack",
    };
    return Some(format!("{scheme}://{absolute_resource_path}"));
  }

  if has_url_scheme(absolute_resource_path) {
    return Some(absolute_resource_path.to_string());
  }

  let Some(source_map_path) = source_map_path else {
    // During the inline source map stage, the asset filename may not be available yet.
    // In that case we cannot compute a relative path and must return None.
    return None;
  };

  // Keep existing identifier expansion; only resource paths opt into
  // component-aware normalization.
  let path_end = if normalize_path && has_suffix {
    resource_path_end(absolute_resource_path)
  } else {
    absolute_resource_path.len()
  };
  let (resource_path, suffix) = absolute_resource_path.split_at(path_end);
  let path = match source_map_path.parent() {
    Some(parent) => Cow::Owned(Path::new(resource_path).relative(parent)),
    None => resource_path.normalize(),
  };
  let path = path.to_string_lossy();
  let mut result = if !normalize_path || cfg!(windows) || is_windows_path(resource_path) {
    path.cow_replace('\\', "/").into_owned()
  } else {
    path.into_owned()
  };
  result.push_str(suffix);
  Some(result)
}

impl ModuleFilenameHelpers {
  fn create_module_filename_template_fn_ctx(
    source_reference: &SourceReference,
    compilation: &Compilation,
    output_options: &OutputOptions,
    namespace: &str,
    unresolved_source_map_path: Option<&Utf8Path>,
  ) -> ModuleFilenameTemplateFnCtx {
    let Compilation { options, .. } = compilation;
    let context = &options.context;

    match source_reference {
      SourceReference::Module(module_identifier) => {
        let module_graph = compilation.get_module_graph();
        let module = module_graph
          .module_by_identifier(module_identifier)
          .unwrap_or_else(|| {
            panic!("failed to find a module for the given identifier '{module_identifier}'")
          });

        let short_identifier = module.readable_identifier(context).to_string();
        let identifier = contextify(context, module_identifier);
        // Readable identifiers may contain display-only prefixes such as `css `.
        let resource_identifier = match module.as_normal_module() {
          Some(normal_module) => contextify(context, normal_module.user_request()),
          None => short_identifier.clone(),
        };
        let module_id =
          ChunkGraph::get_module_id(&compilation.module_ids_artifact, *module_identifier)
            .map_or_default(|s| s.to_string());
        let absolute_resource_path = match module.as_normal_module() {
          Some(normal_module) => normal_module.resource_resolved_data().path().map_or_else(
            || {
              module
                .identifier()
                .split('!')
                .next_back()
                .unwrap_or("")
                .to_string()
            },
            |p| p.to_string(),
          ),
          None => module
            .identifier()
            .split('!')
            .next_back()
            .unwrap_or("")
            .to_string(),
        };

        let hash = get_hash(&identifier, output_options);

        let resource = resource_identifier
          .split('!')
          .next_back()
          .unwrap_or("")
          .to_string();
        let relative_resource_path = resolve_relative_resource_path(
          &absolute_resource_path,
          unresolved_source_map_path,
          options.experiments.runtime_mode,
          should_normalize_source_path(source_reference, compilation),
          false,
        );

        let loaders = get_before(&resource_identifier, "!").to_string();
        let all_loaders = get_before(&identifier, "!").to_string();
        let query = get_after(&resource, "?").to_string();

        let q = query.len();
        let resource_path = if q == 0 {
          resource.clone()
        } else {
          resource[..resource.len().saturating_sub(q)].to_string()
        };

        ModuleFilenameTemplateFnCtx {
          short_identifier,
          identifier,
          module_id,
          absolute_resource_path,
          relative_resource_path,
          hash,
          resource,
          loaders,
          all_loaders,
          query,
          resource_path,
          namespace: namespace.to_string(),
        }
      }
      SourceReference::Source(source) => {
        let short_identifier = contextify(context, source);
        let identifier = short_identifier.clone();

        let hash = get_hash(&identifier, output_options);

        let resource = short_identifier
          .split('!')
          .next_back()
          .unwrap_or("")
          .to_string();

        let loaders = get_before(&short_identifier, "!").to_string();
        let all_loaders = get_before(&identifier, "!").to_string();
        let query = get_after(&resource, "?").to_string();

        let q = query.len();
        let resource_path = if q == 0 {
          resource.clone()
        } else {
          resource[..resource.len().saturating_sub(q)].to_string()
        };

        let absolute_resource_path = source.split('!').next_back().unwrap_or("").to_string();
        let relative_resource_path = resolve_relative_resource_path(
          &absolute_resource_path,
          unresolved_source_map_path,
          options.experiments.runtime_mode,
          should_normalize_source_path(source_reference, compilation),
          true,
        );

        ModuleFilenameTemplateFnCtx {
          short_identifier,
          identifier,
          module_id: String::new(),
          absolute_resource_path,
          relative_resource_path,
          hash,
          resource,
          loaders,
          all_loaders,
          query,
          resource_path,
          namespace: namespace.to_string(),
        }
      }
    }
  }

  pub async fn create_filename_of_fn_template(
    source_reference: &SourceReference,
    compilation: &Compilation,
    module_filename_template: &ModuleFilenameTemplateFn,
    output_options: &OutputOptions,
    namespace: &str,
    unresolved_source_map_path: Option<&Utf8Path>,
  ) -> Result<String> {
    let ctx = ModuleFilenameHelpers::create_module_filename_template_fn_ctx(
      source_reference,
      compilation,
      output_options,
      namespace,
      unresolved_source_map_path,
    );

    module_filename_template(ctx).await
  }

  pub fn create_filename_of_string_template(
    source_reference: &SourceReference,
    compilation: &Compilation,
    module_filename_template: &str,
    output_options: &OutputOptions,
    namespace: &str,
    unresolved_source_map_path: Option<&Utf8Path>,
  ) -> String {
    if let Some(scheme) = default_resource_path_template_scheme(module_filename_template) {
      return create_default_module_filename(source_reference, compilation, namespace, scheme);
    }

    let ctx = ModuleFilenameHelpers::create_module_filename_template_string_ctx(
      source_reference,
      compilation,
      output_options,
      namespace,
      unresolved_source_map_path,
    );

    template_replace(module_filename_template, &ctx)
  }

  pub fn replace_duplicates<F>(filenames: Vec<String>, mut fn_replace: F) -> Vec<String>
  where
    F: FnMut(String, usize, usize) -> String,
  {
    let mut count_map: HashMap<String, Vec<usize>> = HashMap::default();
    let mut pos_map: HashMap<String, usize> = HashMap::default();

    for (idx, item) in filenames.iter().enumerate() {
      count_map.entry(item.clone()).or_default().push(idx);
      pos_map.entry(item.clone()).or_insert(0);
    }

    filenames
      .into_iter()
      .enumerate()
      .map(|(i, item)| {
        let count = count_map
          .get(&item)
          .expect("should have a count entry in count_map");
        if count.len() > 1 {
          let pos = pos_map
            .get_mut(&item)
            .expect("should have a position entry in pos_map");
          let result = fn_replace(item, i, *pos);
          *pos += 1;
          result
        } else {
          item
        }
      })
      .collect()
  }

  fn create_module_filename_template_string_ctx<'a>(
    source_reference: &'a SourceReference,
    compilation: &'a Compilation,
    output_options: &'a OutputOptions,
    namespace: &'a str,
    unresolved_source_map_path: Option<&'a Utf8Path>,
  ) -> ModuleFilenameTemplateStringCtx<'a> {
    ModuleFilenameTemplateStringCtx {
      source_reference,
      compilation,
      output_options,
      namespace,
      unresolved_source_map_path,
      short_identifier: Default::default(),
      identifier: Default::default(),
      resource_identifier: Default::default(),
      normalize_path: Default::default(),
    }
  }
}

fn create_default_module_filename(
  source_reference: &SourceReference,
  compilation: &Compilation,
  namespace: &str,
  scheme: &str,
) -> String {
  let Compilation { options, .. } = compilation;
  let context = &options.context;

  let short_identifier = match source_reference {
    SourceReference::Module(module_identifier) => {
      let module_graph = compilation.get_module_graph();
      let module = module_graph
        .module_by_identifier(module_identifier)
        .unwrap_or_else(|| {
          panic!("failed to find a module for the given identifier '{module_identifier}'")
        });
      module.readable_identifier(context)
    }
    SourceReference::Source(source) => Cow::Owned(contextify(context, source)),
  };

  let resource = short_identifier.split('!').next_back().unwrap_or("");
  let resource_path = split_at_query_mark(resource).0;

  let mut result = String::with_capacity(scheme.len() + namespace.len() + 1 + resource_path.len());
  result.push_str(scheme);
  result.push_str(namespace);
  result.push('/');
  if should_normalize_source_path(source_reference, compilation) {
    push_normalized_resource(&mut result, resource_path);
  } else {
    result.push_str(resource_path);
  }
  result
}

struct ModuleFilenameTemplateStringCtx<'a> {
  source_reference: &'a SourceReference,
  compilation: &'a Compilation,
  output_options: &'a OutputOptions,
  namespace: &'a str,
  unresolved_source_map_path: Option<&'a Utf8Path>,

  // Lazy fields using OnceCell for caching
  short_identifier: OnceCell<Cow<'a, str>>,
  identifier: OnceCell<Cow<'a, str>>,
  resource_identifier: OnceCell<Cow<'a, str>>,
  normalize_path: OnceCell<bool>,
}

impl<'a> ModuleFilenameTemplateStringCtx<'a> {
  fn should_normalize_path(&self) -> bool {
    *self
      .normalize_path
      .get_or_init(|| should_normalize_source_path(self.source_reference, self.compilation))
  }

  fn push_resource_path(&self, buf: &mut String, resource: &str) {
    if self.should_normalize_path() {
      push_normalized_resource(buf, resource);
    } else {
      buf.push_str(resource);
    }
  }

  fn push_absolute_resource_path(&self, buf: &mut String) {
    let path = self.absolute_resource_path();
    if self.should_normalize_path() && matches!(self.source_reference, SourceReference::Module(_)) {
      // NormalModule stores a filesystem path separately from query/fragment.
      if cfg!(windows) || is_windows_path(path) {
        buf.push_str(&path.cow_replace('\\', "/"));
      } else {
        buf.push_str(path);
      }
    } else {
      self.push_resource_path(buf, path);
    }
  }

  pub fn short_identifier(&self) -> &str {
    self.short_identifier.get_or_init(|| {
      let Compilation { options, .. } = self.compilation;
      let context = &options.context;

      match &self.source_reference {
        SourceReference::Module(module_identifier) => {
          let module_graph = self.compilation.get_module_graph();
          let module = module_graph
            .module_by_identifier(module_identifier)
            .unwrap_or_else(|| {
              panic!("failed to find a module for the given identifier '{module_identifier}'")
            });
          module.readable_identifier(context)
        }
        SourceReference::Source(source) => Cow::Owned(contextify(context, source)),
      }
    })
  }

  pub fn identifier(&self) -> &str {
    let Compilation { options, .. } = self.compilation;
    let context = &options.context;

    match &self.source_reference {
      SourceReference::Module(module_identifier) => self
        .identifier
        .get_or_init(|| Cow::Owned(contextify(context, module_identifier))),
      SourceReference::Source(_) => {
        // For Source, identifier is the same as short_identifier
        self.short_identifier()
      }
    }
  }

  pub fn module_id(&self) -> &str {
    match &self.source_reference {
      SourceReference::Module(module_identifier) => {
        ChunkGraph::get_module_id(&self.compilation.module_ids_artifact, *module_identifier)
          .map_or_default(|s| s.as_str())
      }
      SourceReference::Source(_) => "",
    }
  }

  pub fn absolute_resource_path(&self) -> &str {
    match &self.source_reference {
      SourceReference::Module(module_identifier) => {
        let module_graph = self.compilation.get_module_graph();
        if let Some(path) = module_graph
          .module_by_identifier(module_identifier)
          .and_then(|module| module.as_normal_module())
          .and_then(|module| module.resource_resolved_data().path())
        {
          return path.as_str();
        }
        module_identifier.split('!').next_back().unwrap_or("")
      }
      SourceReference::Source(source) => source.split('!').next_back().unwrap_or(""),
    }
  }

  pub fn relative_resource_path(&self) -> Option<Cow<'_, str>> {
    let absolute_resource_path = self.absolute_resource_path();
    let resolved = resolve_relative_resource_path(
      absolute_resource_path,
      self.unresolved_source_map_path,
      self.compilation.options.experiments.runtime_mode,
      self.should_normalize_path(),
      matches!(self.source_reference, SourceReference::Source(_)),
    );

    match (resolved, &self.source_reference) {
      (Some(path), _) => Some(Cow::Owned(path)),
      (None, SourceReference::Module(_)) => {
        // Eval devtool modes have no source map file path to resolve against.
        let resource_identifier = self.resource_identifier();
        let resource = resource_identifier.split('!').next_back().unwrap_or("");
        Some(Cow::Owned(resource.to_string()))
      }
      (None, SourceReference::Source(_)) => None,
    }
  }

  pub fn hash(&self) -> String {
    let identifier = self.identifier();
    get_hash(identifier, self.output_options)
  }

  pub fn resource(&self) -> &str {
    let resource_identifier = self.resource_identifier();
    resource_identifier.split('!').next_back().unwrap_or("")
  }

  pub fn loaders(&self) -> &str {
    let resource_identifier = self.resource_identifier();
    get_before(resource_identifier, "!")
  }

  /// Returns the request path without readable-identifier decorations such as `css `.
  pub fn resource_identifier(&self) -> &str {
    self.resource_identifier.get_or_init(|| {
      let Compilation { options, .. } = self.compilation;
      let context = &options.context;

      match &self.source_reference {
        SourceReference::Module(module_identifier) => {
          let module_graph = self.compilation.get_module_graph();
          let module = module_graph
            .module_by_identifier(module_identifier)
            .unwrap_or_else(|| {
              panic!("failed to find a module for the given identifier '{module_identifier}'")
            });
          match module.as_normal_module() {
            Some(normal_module) => Cow::Owned(contextify(context, normal_module.user_request())),
            None => module.readable_identifier(context),
          }
        }
        SourceReference::Source(source) => Cow::Owned(contextify(context, source)),
      }
    })
  }

  pub fn all_loaders(&self) -> &str {
    let identifier = self.identifier();
    get_before(identifier, "!")
  }

  pub fn query(&self) -> &str {
    let resource = self.resource();
    get_after(resource, "?")
  }

  pub fn resource_path(&self) -> &str {
    let resource = self.resource();
    let query = self.query();
    let q = query.len();
    if q == 0 {
      resource
    } else {
      &resource[..resource.len().saturating_sub(q)]
    }
  }

  pub fn namespace(&self) -> &str {
    self.namespace
  }
}

fn starts_with_ignore_ascii_case(s: &[u8], prefix: &[u8]) -> bool {
  s.len() >= prefix.len() && s[..prefix.len()].eq_ignore_ascii_case(prefix)
}

fn template_replace<'a>(s: &str, ctx: &ModuleFilenameTemplateStringCtx<'a>) -> String {
  let resource_tag = b"[resource]";
  let sstr = s;
  let s = s.as_bytes();
  let mut buf = String::new();
  let mut pos = 0;
  let mut state = false;

  macro_rules! match_ignore_case {
        (
            $value:expr ;
            $(
                $item:literal $( | $item2:literal )* => $b:expr,
            )*
            $name:ident => $tail:expr
        ) => {
            $(
                if $value.eq_ignore_ascii_case($item)
                    $( || $value.eq_ignore_ascii_case($item2) )*
                {
                    $b
                } else
            )*

            {
                let $name = $value;
                $tail
            }
        }
    }

  for i in memchr::memchr2_iter(b'[', b']', s) {
    if i < pos {
      continue;
    }

    match s[i] {
      b'[' => {
        // # Safety
        //
        // always utf8
        let s = &sstr[pos..i];
        buf.push_str(s);
        pos = i;
        state = true;
      }
      b']' if state => {
        let mut next_pos = i + 1;
        match_ignore_case!(&s[pos..next_pos];
            b"[identifier]" => buf.push_str(ctx.identifier()),
            b"[short-identifier]" => buf.push_str(ctx.short_identifier()),
            b"[resource]" => ctx.push_resource_path(&mut buf, ctx.resource()),
            b"[resource-path]" |  b"[resourcepath]" => ctx.push_resource_path(&mut buf, ctx.resource_path()),

            b"[absolute-resource-path]" |
            b"[abs-resource-path]" |
            b"[absoluteresource-path]" |
            b"[absresource-path]" |
            b"[absolute-resourcepath]" |
            b"[abs-resourcepath]" |
            b"[absoluteresourcepath]" |
            b"[absresourcepath]" => ctx.push_absolute_resource_path(&mut buf),

            b"[relative-resource-path]" |
            b"[relativeresource-path]" |
            b"[relative-resourcepath]" |
            b"[relativeresourcepath]" => {
              if let Some(relative_resource_path) = ctx.relative_resource_path() {
                ctx.push_resource_path(&mut buf, relative_resource_path.as_ref())
              } else {
                buf.push_str(&sstr[pos..next_pos]);
              }
            },

            b"[all-loaders]" | b"[allloaders]" => if starts_with_ignore_ascii_case(&s[next_pos..], resource_tag) {
                next_pos += resource_tag.len();
                buf.push_str(ctx.identifier());
            } else {
                buf.push_str(ctx.all_loaders());
            },
            b"[loaders]" => if starts_with_ignore_ascii_case(&s[next_pos..], resource_tag) {
                next_pos += resource_tag.len();
                buf.push_str(ctx.short_identifier());
            } else {
                buf.push_str(ctx.loaders());
            },

            b"[query]" => buf.push_str(ctx.query()),
            b"[id]" => buf.push_str(ctx.module_id()),
            b"[hash]" => buf.push_str(ctx.hash().as_ref()),
            b"[namespace]" => buf.push_str(ctx.namespace()),

            matched => if let Some(matched) = matched.strip_prefix(b"[\\")
                .and_then(|matched| matched.strip_suffix(b"\\]"))
            {
                // # Safety
                //
                // always utf8
                #[allow(clippy::unwrap_used)]
                let s = str::from_utf8(matched).unwrap();
                buf.push('[');
                buf.push_str(s);
                buf.push(']');
            } else {
                // # Safety
                //
                // always utf8
                let s = &sstr[pos..next_pos];
                buf.push_str(s);
            }
        );

        pos = next_pos;
        state = false;
      }
      _ => (),
    }
  }

  // # Safety
  //
  // always utf8
  let s = &sstr[pos..];
  buf.push_str(s);
  buf
}
