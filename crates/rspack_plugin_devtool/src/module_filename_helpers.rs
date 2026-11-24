use std::{borrow::Cow, cell::OnceCell, path::Path, sync::LazyLock};

use cow_utils::CowUtils;
use regex::Regex;
use rspack_core::{ChunkGraph, Compilation, OutputOptions, contextify, runtime_mode::RuntimeMode};
use rspack_error::Result;
use rspack_hash::{RspackHash, RspackHasher};
use rspack_paths::Utf8Path;
use rspack_util::{
  identifier::{absolute_to_request, split_at_query_mark, split_resource},
  node_path::NodePath,
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

fn is_windows_path(path: &str) -> bool {
  path.starts_with(r"\\")
    || (path.as_bytes().get(1) == Some(&b':')
      && !path.starts_with(['/', '\\'])
      && Utf8Path::new(path).node_is_absolute_win32())
}

fn is_source_url(source: &str) -> bool {
  starts_with_url_scheme(source) && !is_windows_path(source)
}

pub(super) fn split_source_request(request: &str) -> (&str, &str) {
  // The shared resource parser skips the DOS prefix on Windows. A loader map
  // can contain the same prefix on a different host.
  let body = if cfg!(windows) {
    request
  } else {
    request.strip_prefix(r"\\?\").unwrap_or(request)
  };
  let (path, _, _) = split_resource(body);
  request.split_at(request.len() - body.len() + path.len())
}

fn shorten_source_path<'a>(context: &str, source: &'a str) -> Cow<'a, str> {
  if is_source_url(source) || (!cfg!(windows) && is_windows_path(source)) {
    return Cow::Borrowed(source);
  }
  let (path, suffix) = split_source_request(source);
  match absolute_to_request(context, path) {
    Cow::Borrowed(_) => Cow::Borrowed(source),
    Cow::Owned(mut request) => {
      request.push_str(suffix);
      Cow::Owned(request)
    }
  }
}

enum TemplateResource<'a> {
  Request(&'a str),
  ResolvedPath(&'a str),
  Identifier(&'a str),
}

impl<'a> TemplateResource<'a> {
  fn value(&self) -> &'a str {
    match *self {
      Self::Request(value) | Self::ResolvedPath(value) | Self::Identifier(value) => value,
    }
  }

  fn path_and_suffix(&self) -> Option<(&'a str, &'a str)> {
    match *self {
      Self::Request(value) if !is_source_url(value) => Some(split_source_request(value)),
      Self::ResolvedPath(value) if !is_source_url(value) => Some((value, "")),
      _ => None,
    }
  }

  fn write(&self, buf: &mut String, include_query: bool) {
    if let Some((path, suffix)) = self.path_and_suffix() {
      let suffix = if include_query || !suffix.starts_with('?') {
        suffix
      } else {
        ""
      };
      write_source_path(buf, path, suffix, cfg!(windows) || is_windows_path(path));
    } else if let Self::Request(url) = self
      && !include_query
    {
      buf.push_str(split_at_query_mark(url).0);
    } else {
      buf.push_str(self.value());
    }
  }
}

fn write_source_path(buf: &mut String, path: &str, suffix: &str, windows: bool) {
  if windows {
    buf.push_str(&path.cow_replace('\\', "/"));
  } else {
    buf.push_str(path);
  }
  buf.push_str(suffix);
}

fn runtime_source_url(source: &str, runtime_mode: RuntimeMode) -> Option<String> {
  if source.starts_with("webpack/runtime")
    || source.starts_with("rspack/runtime")
    || matches!(
      source,
      "webpack/bootstrap" | "webpack/startup" | "rspack/bootstrap" | "rspack/startup"
    )
  {
    // Runtime modules, bootstrap, and startup are virtual.
    let scheme = match runtime_mode {
      RuntimeMode::Webpack => "webpack",
      RuntimeMode::Rspack => "rspack",
    };
    return Some(format!("{scheme}://{source}"));
  }

  None
}

fn resolve_relative_resource_path(
  resource: TemplateResource<'_>,
  source_map_path: Option<&Utf8Path>,
  runtime_mode: RuntimeMode,
) -> Option<String> {
  if let Some(url) = runtime_source_url(resource.value(), runtime_mode) {
    return Some(url);
  }

  let Some((path, suffix)) = resource.path_and_suffix() else {
    return Some(resource.value().to_string());
  };
  let map_path = source_map_path?;
  let windows = cfg!(windows) || is_windows_path(path);
  let relative = if windows && !cfg!(windows) {
    // A path from another platform cannot be made relative to this map.
    Cow::Borrowed(path)
  } else if let Some(parent) = map_path.parent() {
    Cow::Owned(
      Path::new(path)
        .relative(parent)
        .to_string_lossy()
        .into_owned(),
    )
  } else {
    Cow::Owned(Utf8Path::new(path).node_normalize().into_string())
  };
  let mut result = String::with_capacity(relative.len() + suffix.len());
  write_source_path(&mut result, &relative, suffix, windows);
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
        let absolute_resource = match module
          .as_normal_module()
          .and_then(|module| module.resource_resolved_data().path())
        {
          Some(path) => TemplateResource::ResolvedPath(path.as_str()),
          None => {
            TemplateResource::Identifier(module_identifier.rsplit('!').next().unwrap_or_default())
          }
        };
        let absolute_resource_path = absolute_resource.value().to_string();

        let hash = get_hash(&identifier, output_options);

        let resource = resource_identifier
          .split('!')
          .next_back()
          .unwrap_or("")
          .to_string();
        let relative_resource_path = resolve_relative_resource_path(
          absolute_resource,
          unresolved_source_map_path,
          options.experiments.runtime_mode,
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
        let short_identifier = shorten_source_path(context.as_str(), source).into_owned();
        let identifier = short_identifier.clone();
        let hash = get_hash(&identifier, output_options);
        let (path, suffix) = split_source_request(&short_identifier);
        let query = suffix
          .starts_with('?')
          .then_some(suffix)
          .unwrap_or_default();
        let resource_path = if query.is_empty() {
          short_identifier.clone()
        } else {
          path.to_string()
        };
        let query = query.to_string();
        let relative_resource_path = resolve_relative_resource_path(
          TemplateResource::Request(source),
          unresolved_source_map_path,
          options.experiments.runtime_mode,
        );
        ModuleFilenameTemplateFnCtx {
          short_identifier: short_identifier.clone(),
          identifier,
          module_id: String::new(),
          absolute_resource_path: source.to_string(),
          relative_resource_path,
          hash,
          resource: short_identifier,
          loaders: String::new(),
          all_loaders: String::new(),
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

  let request;
  let resource = match source_reference {
    SourceReference::Module(module_identifier) => {
      let module = compilation
        .get_module_graph()
        .module_by_identifier(module_identifier)
        .unwrap_or_else(|| {
          panic!("failed to find a module for the given identifier '{module_identifier}'")
        });
      request = module.readable_identifier(context);
      if module
        .as_normal_module()
        .and_then(|module| module.resource_resolved_data().path())
        .is_some()
      {
        TemplateResource::Request(request.rsplit('!').next().unwrap_or_default())
      } else {
        TemplateResource::Identifier(&request)
      }
    }
    SourceReference::Source(source) => {
      request = shorten_source_path(context.as_str(), source);
      TemplateResource::Request(&request)
    }
  };

  let mut result =
    String::with_capacity(scheme.len() + namespace.len() + 1 + resource.value().len());
  result.push_str(scheme);
  result.push_str(namespace);
  result.push('/');
  resource.write(&mut result, false);
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
}

impl<'a> ModuleFilenameTemplateStringCtx<'a> {
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
        SourceReference::Source(source) => shorten_source_path(context.as_str(), source),
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

  fn resolved_module_path(&self) -> Option<&str> {
    let SourceReference::Module(identifier) = self.source_reference else {
      return None;
    };
    self
      .compilation
      .get_module_graph()
      .module_by_identifier(identifier)
      .and_then(|module| module.as_normal_module())
      .and_then(|module| module.resource_resolved_data().path())
      .map(Utf8Path::as_str)
  }

  fn absolute_resource(&self) -> TemplateResource<'_> {
    match self.source_reference {
      SourceReference::Module(identifier) => match self.resolved_module_path() {
        Some(path) => TemplateResource::ResolvedPath(path),
        None => TemplateResource::Identifier(identifier.rsplit('!').next().unwrap_or_default()),
      },
      SourceReference::Source(source) => TemplateResource::Request(source),
    }
  }

  fn relative_resource_path(&self) -> Option<String> {
    let relative = resolve_relative_resource_path(
      self.absolute_resource(),
      self.unresolved_source_map_path,
      self.compilation.options.experiments.runtime_mode,
    );
    if relative.is_none() && matches!(self.source_reference, SourceReference::Module(_)) {
      // Eval source maps do not have a filename to resolve against.
      let mut result = String::new();
      self.resource().write(&mut result, true);
      return Some(result);
    }
    relative
  }

  pub fn hash(&self) -> String {
    let identifier = self.identifier();
    get_hash(identifier, self.output_options)
  }

  fn resource(&self) -> TemplateResource<'_> {
    match self.source_reference {
      SourceReference::Source(_) => TemplateResource::Request(self.short_identifier()),
      SourceReference::Module(_) if self.resolved_module_path().is_some() => {
        TemplateResource::Request(
          self
            .resource_identifier()
            .rsplit('!')
            .next()
            .unwrap_or_default(),
        )
      }
      SourceReference::Module(_) => TemplateResource::Identifier(self.resource_identifier()),
    }
  }

  pub fn loaders(&self) -> &str {
    match self.source_reference {
      SourceReference::Module(_) => get_before(self.resource_identifier(), "!"),
      SourceReference::Source(_) => "",
    }
  }

  /// Returns the request path without readable-identifier decorations such as `css `.
  pub fn resource_identifier(&self) -> &str {
    let SourceReference::Module(module_identifier) = self.source_reference else {
      return self.short_identifier();
    };
    self.resource_identifier.get_or_init(|| {
      let context = &self.compilation.options.context;
      let module = self
        .compilation
        .get_module_graph()
        .module_by_identifier(module_identifier)
        .unwrap_or_else(|| {
          panic!("failed to find a module for the given identifier '{module_identifier}'")
        });
      match module.as_normal_module() {
        Some(normal_module) => Cow::Owned(contextify(context, normal_module.user_request())),
        None => module.readable_identifier(context),
      }
    })
  }

  pub fn all_loaders(&self) -> &str {
    match self.source_reference {
      SourceReference::Module(_) => get_before(self.identifier(), "!"),
      SourceReference::Source(_) => "",
    }
  }

  pub fn query(&self) -> &str {
    match self.resource() {
      TemplateResource::Request(request) => {
        let (_, suffix) = split_source_request(request);
        if suffix.starts_with('?') { suffix } else { "" }
      }
      _ => "",
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
            b"[identifier]" => buf.push_str(ctx.identifier().as_ref()),
            b"[short-identifier]" => buf.push_str(ctx.short_identifier().as_ref()),
            b"[resource]" => ctx.resource().write(&mut buf, true),
            b"[resource-path]" |  b"[resourcepath]" => ctx.resource().write(&mut buf, false),

            b"[absolute-resource-path]" |
            b"[abs-resource-path]" |
            b"[absoluteresource-path]" |
            b"[absresource-path]" |
            b"[absolute-resourcepath]" |
            b"[abs-resourcepath]" |
            b"[absoluteresourcepath]" |
            b"[absresourcepath]" => ctx.absolute_resource().write(&mut buf, true),

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
                buf.push_str(ctx.identifier().as_ref());
            } else {
                buf.push_str(ctx.all_loaders());
            },
            b"[loaders]" => if starts_with_ignore_ascii_case(&s[next_pos..], resource_tag) {
                next_pos += resource_tag.len();
                buf.push_str(ctx.short_identifier().as_ref());
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
