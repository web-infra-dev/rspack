use std::{borrow::Cow, cell::OnceCell, path::Path, sync::LazyLock};

use cow_utils::CowUtils;
use regex::Regex;
use rspack_core::{ChunkGraph, Compilation, OutputOptions, contextify, runtime_mode::RuntimeMode};
use rspack_error::Result;
use rspack_hash::{RspackHash, RspackHasher};
use rspack_paths::Utf8Path;
use rspack_util::{
  identifier::{split_at_query_mark, split_resource_path},
  node_path,
};
use rustc_hash::FxHashMap as HashMap;
use sugar_path::SugarPath;

use crate::{
  ModuleFilenameTemplateFn, ModuleFilenameTemplateFnCtx, SourceReference,
  source_map_dev_tool_plugin::starts_with_url_scheme,
};

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

fn is_explicit_windows_path(path: &str) -> bool {
  // A single leading backslash can be part of a POSIX file name.
  path.starts_with(r"\\") || (!path.starts_with(['/', '\\']) && node_path::is_absolute_win32(path))
}

fn has_url_scheme(resource: &str) -> bool {
  starts_with_url_scheme(resource) && !node_path::is_absolute_win32(resource)
}

enum TemplateResource<'a> {
  Path { path: &'a str, suffix: &'a str },
  Opaque(&'a str),
}

impl<'a> TemplateResource<'a> {
  fn from_request(request: &'a str, is_resource: bool) -> Self {
    if !is_resource || has_url_scheme(request) {
      return Self::Opaque(request);
    }
    let (path, suffix) =
      split_resource_path(request, cfg!(windows) || is_explicit_windows_path(request));
    Self::Path { path, suffix }
  }

  fn without_query(self) -> Self {
    match self {
      Self::Path { path, suffix } if suffix.starts_with('?') => Self::Path { path, suffix: "" },
      Self::Opaque(value) => Self::Opaque(split_at_query_mark(value).0),
      _ => self,
    }
  }

  fn push_to(self, out: &mut String) {
    match self {
      Self::Path { path, suffix } => {
        let windows = cfg!(windows) || is_explicit_windows_path(path);
        push_path(out, path, suffix, windows);
      }
      Self::Opaque(value) => out.push_str(value),
    }
  }
}

fn push_path(out: &mut String, path: &str, suffix: &str, windows: bool) {
  if windows {
    out.push_str(&path.cow_replace('\\', "/"));
  } else {
    out.push_str(path);
  }
  out.push_str(suffix);
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

fn absolute_resource<'a>(
  source_reference: &'a SourceReference,
  compilation: &'a Compilation,
) -> (&'a str, TemplateResource<'a>) {
  match source_reference {
    SourceReference::Module(identifier) => {
      if let Some(path) = compilation
        .get_module_graph()
        .module_by_identifier(identifier)
        .and_then(|module| module.as_normal_module())
        .and_then(|module| module.resource_resolved_data().path())
      {
        let path = path.as_str();
        let resource = if has_url_scheme(path) {
          TemplateResource::Opaque(path)
        } else {
          // Resolved paths already have query and fragment stored separately.
          TemplateResource::Path { path, suffix: "" }
        };
        return (path, resource);
      }
      let value = identifier.rsplit('!').next().unwrap_or_default();
      (value, TemplateResource::Opaque(value))
    }
    SourceReference::Source(source) => {
      let value = source.rsplit('!').next().unwrap_or_default();
      (
        value,
        TemplateResource::from_request(value, !source.contains(['|', '!'])),
      )
    }
  }
}

fn resolve_relative_resource_path(
  absolute: &str,
  resource: TemplateResource<'_>,
  source_map_path: Option<&Utf8Path>,
  runtime_mode: RuntimeMode,
) -> Option<String> {
  if absolute.starts_with("webpack/runtime")
    || absolute.starts_with("rspack/runtime")
    || matches!(
      absolute,
      "webpack/bootstrap" | "webpack/startup" | "rspack/bootstrap" | "rspack/startup"
    )
  {
    let scheme = match runtime_mode {
      RuntimeMode::Webpack => "webpack",
      RuntimeMode::Rspack => "rspack",
    };
    return Some(format!("{scheme}://{absolute}"));
  }

  if has_url_scheme(absolute) {
    return Some(absolute.to_string());
  }

  // Eval source maps may not have a map filename to resolve against.
  let source_map_path = source_map_path?;
  let relative = |path: &str| match source_map_path.parent() {
    Some(parent) => Path::new(path)
      .relative(parent)
      .to_string_lossy()
      .into_owned(),
    None => node_path::normalize(path),
  };

  match resource {
    TemplateResource::Path { path, suffix } => {
      let windows = cfg!(windows) || is_explicit_windows_path(path);
      let relative = if windows && !cfg!(windows) {
        // The POSIX map directory cannot serve as a base for a Windows path.
        Cow::Borrowed(path)
      } else {
        Cow::Owned(relative(path))
      };
      let mut result = String::with_capacity(relative.len() + suffix.len());
      push_path(&mut result, &relative, suffix, windows);
      Some(result)
    }
    TemplateResource::Opaque(value) => Some(relative(value).cow_replace('\\', "/").into_owned()),
  }
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

    let (absolute, resource) = absolute_resource(source_reference, compilation);
    let absolute_resource_path = absolute.to_string();
    let relative_resource_path = resolve_relative_resource_path(
      absolute,
      resource,
      unresolved_source_map_path,
      options.experiments.runtime_mode,
    );

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
        let hash = get_hash(&identifier, output_options);

        let resource = resource_identifier
          .split('!')
          .next_back()
          .unwrap_or("")
          .to_string();
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
  let mut result = String::with_capacity(scheme.len() + namespace.len() + 1 + resource.len());
  result.push_str(scheme);
  result.push_str(namespace);
  result.push('/');
  TemplateResource::from_request(
    resource,
    should_normalize_source_path(source_reference, compilation),
  )
  .without_query()
  .push_to(&mut result);
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

  pub fn relative_resource_path(&self) -> Option<String> {
    let (absolute, resource) = absolute_resource(self.source_reference, self.compilation);
    let resolved = resolve_relative_resource_path(
      absolute,
      resource,
      self.unresolved_source_map_path,
      self.compilation.options.experiments.runtime_mode,
    );
    match (resolved, self.source_reference) {
      (Some(path), _) => Some(path),
      // Eval devtool modes use the module's resource when there is no map filename.
      (None, SourceReference::Module(_)) => {
        let mut result = String::new();
        self.template_resource().push_to(&mut result);
        Some(result)
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

  fn template_resource(&self) -> TemplateResource<'_> {
    TemplateResource::from_request(self.resource(), self.should_normalize_path())
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
            b"[resource]" => ctx.template_resource().push_to(&mut buf),
            b"[resource-path]" |  b"[resourcepath]" => ctx.template_resource().without_query().push_to(&mut buf),

            b"[absolute-resource-path]" |
            b"[abs-resource-path]" |
            b"[absoluteresource-path]" |
            b"[absresource-path]" |
            b"[absolute-resourcepath]" |
            b"[abs-resourcepath]" |
            b"[absoluteresourcepath]" |
            b"[absresourcepath]" => absolute_resource(ctx.source_reference, ctx.compilation).1.push_to(&mut buf),

            b"[relative-resource-path]" |
            b"[relativeresource-path]" |
            b"[relative-resourcepath]" |
            b"[relativeresourcepath]" => {
              if let Some(relative_resource_path) = ctx.relative_resource_path() {
                buf.push_str(&relative_resource_path)
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
