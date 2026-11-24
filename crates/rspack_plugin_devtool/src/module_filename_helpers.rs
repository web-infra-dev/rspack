use std::{borrow::Cow, cell::OnceCell, path::Path, sync::LazyLock};

use cow_utils::CowUtils;
use regex::Regex;
use rspack_core::{
  ChunkGraph, Compilation, ModuleRef, OutputOptions, ResourceData, contextify,
  runtime_mode::RuntimeMode,
};
use rspack_error::Result;
use rspack_hash::{RspackHash, RspackHasher};
use rspack_paths::Utf8Path;
use rspack_util::{
  identifier::{absolute_to_request, split_resource},
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
  if source.starts_with("//")
    || is_source_url(source)
    || (!cfg!(windows) && is_windows_path(source))
  {
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

fn normalize_source_path(path: &str) -> String {
  if cfg!(windows) || is_windows_path(path) {
    // A verbatim drive or UNC path may have a regular Win32 spelling.
    #[cfg(windows)]
    let path = rspack_paths::strip_dos_device_path_prefix(path);
    #[cfg(windows)]
    let path = path.as_ref();

    // Other device namespaces have no slash-separated filesystem spelling.
    if path.starts_with(r"\\?\") || path.starts_with(r"\\.\") {
      return path.to_string();
    }
    Utf8Path::new(path)
      .node_normalize_win32()
      .as_str()
      .cow_replace('\\', "/")
      .into_owned()
  } else {
    Utf8Path::new(path).node_normalize_posix().into_string()
  }
}

fn append_suffixes<'a>(mut path: Cow<'a, str>, suffixes: [&str; 2]) -> Cow<'a, str> {
  for suffix in suffixes {
    if !suffix.is_empty() {
      path.to_mut().push_str(suffix);
    }
  }
  path
}

fn source_query(request: &str) -> &str {
  let (_, suffix) = split_source_request(request);
  if suffix.starts_with('?') { suffix } else { "" }
}

fn is_foreign_windows_path(path: &str) -> bool {
  !cfg!(windows) && (is_windows_path(path) || path.starts_with("//"))
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

fn relative_source_path<'a>(path: &'a str, base: Option<&Utf8Path>) -> Cow<'a, str> {
  let Some(base) = base else {
    return Cow::Borrowed(path);
  };
  if is_foreign_windows_path(path) {
    return Cow::Borrowed(path);
  }
  let relative = Path::new(path).relative(base);
  let relative = relative.to_string_lossy();
  if cfg!(windows) {
    Cow::Owned(relative.cow_replace('\\', "/").into_owned())
  } else {
    Cow::Owned(relative.into_owned())
  }
}

fn source_url(scheme: &str, namespace: &str, name: &str) -> String {
  let mut result = String::with_capacity(scheme.len() + namespace.len() + 1 + name.len());
  result.push_str(scheme);
  result.push_str(namespace);
  result.push('/');
  result.push_str(name);
  result
}

impl ModuleFilenameHelpers {
  pub async fn create_filename_of_fn_template(
    source_reference: &SourceReference,
    compilation: &Compilation,
    module_filename_template: &ModuleFilenameTemplateFn,
    output_options: &OutputOptions,
    namespace: &str,
    unresolved_source_map_path: Option<&Utf8Path>,
  ) -> Result<String> {
    let ctx = ModuleFilenameTemplateCtx::new(
      source_reference,
      compilation,
      output_options,
      namespace,
      unresolved_source_map_path,
    )
    .to_fn_ctx();

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
    let ctx = ModuleFilenameTemplateCtx::new(
      source_reference,
      compilation,
      output_options,
      namespace,
      unresolved_source_map_path,
    );
    if let Some(scheme) = default_resource_path_template_scheme(module_filename_template) {
      return source_url(scheme, namespace, &ctx.resource(false));
    }

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
}

struct ModuleFilenameTemplateCtx<'a> {
  source_reference: &'a SourceReference,
  compilation: &'a Compilation,
  module: Option<&'a ModuleRef>,
  resolved_resource: Option<(&'a ResourceData, &'a str)>,
  file_source: Option<(String, [&'a str; 2])>,
  output_options: &'a OutputOptions,
  namespace: &'a str,
  unresolved_source_map_path: Option<&'a Utf8Path>,

  short_identifier: OnceCell<Cow<'a, str>>,
  identifier: OnceCell<Cow<'a, str>>,
  resource_identifier: OnceCell<Cow<'a, str>>,
}

impl<'a> ModuleFilenameTemplateCtx<'a> {
  fn new(
    source_reference: &'a SourceReference,
    compilation: &'a Compilation,
    output_options: &'a OutputOptions,
    namespace: &'a str,
    unresolved_source_map_path: Option<&'a Utf8Path>,
  ) -> Self {
    let module = match source_reference {
      SourceReference::Module(identifier) => Some(
        compilation
          .get_module_graph()
          .module_by_identifier(identifier)
          .unwrap_or_else(|| {
            panic!("failed to find a module for the given identifier '{identifier}'")
          }),
      ),
      SourceReference::Source(_) | SourceReference::LoaderRequest(_) => None,
    };
    let resolved_resource = module
      .and_then(|module| module.as_normal_module())
      .and_then(|module| {
        let data = module.resource_resolved_data().as_ref();
        Some((data, data.path()?.as_str()))
      });
    // Prepare the path separately from its query and fragment so that
    // templates can reuse it without interpreting the module identifier.
    let file_source = match source_reference {
      SourceReference::Module(_) => resolved_resource.map(|(data, path)| {
        (
          path,
          [
            data.query().unwrap_or_default(),
            data.fragment().unwrap_or_default(),
          ],
        )
      }),
      _ => source_reference
        .resource_request()
        .filter(|source| !source.starts_with("//"))
        .map(|source| {
          let (path, suffix) = split_source_request(source);
          (path, [suffix, ""])
        }),
    }
    .filter(|(path, _)| !path.is_empty() && !is_source_url(path))
    .map(|(path, suffixes)| (normalize_source_path(path), suffixes));
    Self {
      source_reference,
      compilation,
      module,
      resolved_resource,
      file_source,
      output_options,
      namespace,
      unresolved_source_map_path,
      short_identifier: Default::default(),
      identifier: Default::default(),
      resource_identifier: Default::default(),
    }
  }

  fn to_fn_ctx(&self) -> ModuleFilenameTemplateFnCtx {
    let (resource, query) = if !matches!(self.source_reference, SourceReference::Source(_)) {
      let identifier = self.resource_identifier();
      let resource = identifier.rsplit('!').next().unwrap_or_default();
      let query = resource.find('?').map_or("", |start| &resource[start..]);
      (resource, query)
    } else {
      let resource = self.short_identifier();
      (resource, source_query(resource))
    };
    let relative_resource_path = if self.is_virtual_module() {
      runtime_source_url(
        self.raw_absolute_resource_path(),
        self.compilation.options.experiments.runtime_mode,
      )
      .or_else(|| {
        self.unresolved_source_map_path?;
        Some(self.virtual_source_url(self.short_identifier()))
      })
    } else {
      self.relative_resource_path().map(Cow::into_owned)
    };
    ModuleFilenameTemplateFnCtx {
      short_identifier: self.short_identifier().to_string(),
      identifier: self.identifier().to_string(),
      absolute_resource_path: self.raw_absolute_resource_path().to_string(),
      relative_resource_path,
      resource: resource.to_string(),
      resource_path: resource.strip_suffix(query).unwrap_or(resource).to_string(),
      loaders: self.loaders().to_string(),
      all_loaders: self.all_loaders().to_string(),
      query: query.to_string(),
      module_id: self.module_id().to_string(),
      hash: self.hash(),
      namespace: self.namespace.to_string(),
    }
  }

  pub fn short_identifier(&self) -> &str {
    self.short_identifier.get_or_init(|| {
      let Compilation { options, .. } = self.compilation;
      let context = &options.context;

      match self.source_reference {
        SourceReference::Module(_) => self
          .module
          .expect("module already resolved")
          .readable_identifier(context),
        SourceReference::Source(source) => shorten_source_path(context.as_str(), source),
        SourceReference::LoaderRequest(request) => Cow::Owned(contextify(context, request)),
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
      SourceReference::Source(_) | SourceReference::LoaderRequest(_) => {
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
      SourceReference::Source(_) | SourceReference::LoaderRequest(_) => "",
    }
  }

  fn is_virtual_module(&self) -> bool {
    self.module.is_some() && self.resolved_resource.is_none()
  }

  fn virtual_source_url(&self, name: &str) -> String {
    let scheme = match self.compilation.options.experiments.runtime_mode {
      RuntimeMode::Webpack => "webpack://",
      RuntimeMode::Rspack => "rspack://",
    };
    source_url(scheme, self.namespace, name)
  }

  fn raw_absolute_resource_path(&self) -> &str {
    match self.source_reference {
      SourceReference::Module(identifier) => self.resolved_resource.map_or_else(
        || identifier.rsplit('!').next().unwrap_or_default(),
        |(_, path)| path,
      ),
      _ => self.source_reference.resource_request().unwrap_or_default(),
    }
  }

  fn absolute_resource_path(&self) -> Cow<'_, str> {
    if let Some((path, suffixes)) = &self.file_source {
      return append_suffixes(Cow::Borrowed(path), *suffixes);
    }
    Cow::Borrowed(self.resource_name())
  }

  fn relative_resource_path(&self) -> Option<Cow<'_, str>> {
    if self.is_virtual_module() {
      return Some(Cow::Borrowed(self.short_identifier()));
    }
    if let Some(url) = runtime_source_url(
      self.raw_absolute_resource_path(),
      self.compilation.options.experiments.runtime_mode,
    ) {
      return Some(Cow::Owned(url));
    }
    let Some((path, suffixes)) = &self.file_source else {
      return Some(Cow::Borrowed(self.resource_name()));
    };
    let map_path = self.unresolved_source_map_path?;
    Some(append_suffixes(
      relative_source_path(path, map_path.parent()),
      *suffixes,
    ))
  }

  pub fn hash(&self) -> String {
    let identifier = self.identifier();
    get_hash(identifier, self.output_options)
  }

  fn resource_name(&self) -> &str {
    self.resolved_resource.map_or_else(
      || match self.source_reference {
        SourceReference::Module(_) => self.short_identifier(),
        _ => self.source_reference.resource_request().unwrap_or_default(),
      },
      |(data, _)| data.resource(),
    )
  }

  fn resource(&self, include_query: bool) -> Cow<'_, str> {
    if let Some((path, suffixes)) = &self.file_source {
      let request = if is_foreign_windows_path(path) {
        Cow::Borrowed(path.as_str())
      } else {
        absolute_to_request(self.compilation.options.context.as_str(), path)
      };
      return if include_query || !suffixes[0].starts_with('?') {
        append_suffixes(request, *suffixes)
      } else {
        request
      };
    }
    let resource = self.resource_name();
    let query = if include_query || self.is_virtual_module() {
      ""
    } else {
      source_query(resource)
    };
    Cow::Borrowed(&resource[..resource.len() - query.len()])
  }

  pub fn loaders(&self) -> &str {
    match self.source_reference {
      SourceReference::Source(_) => "",
      _ => get_before(self.resource_identifier(), "!"),
    }
  }

  /// Returns the request path without readable-identifier decorations such as `css `.
  pub fn resource_identifier(&self) -> &str {
    let Some(module) = self.module else {
      return self.short_identifier();
    };
    self.resource_identifier.get_or_init(|| {
      let context = &self.compilation.options.context;
      match module.as_normal_module() {
        Some(normal_module) => Cow::Owned(contextify(context, normal_module.user_request())),
        None => module.readable_identifier(context),
      }
    })
  }

  pub fn all_loaders(&self) -> &str {
    match self.source_reference {
      SourceReference::Source(_) => "",
      _ => get_before(self.identifier(), "!"),
    }
  }

  pub fn query(&self) -> Cow<'_, str> {
    if let Some((_, [query, fragment])) = &self.file_source {
      if query.starts_with('?') {
        return append_suffixes(Cow::Borrowed(*query), [fragment, ""]);
      }
      return Cow::Borrowed("");
    }
    if self.is_virtual_module() {
      Cow::Borrowed("")
    } else {
      Cow::Borrowed(source_query(self.resource_name()))
    }
  }

  pub fn namespace(&self) -> &str {
    self.namespace
  }
}

fn starts_with_ignore_ascii_case(s: &[u8], prefix: &[u8]) -> bool {
  s.len() >= prefix.len() && s[..prefix.len()].eq_ignore_ascii_case(prefix)
}

fn template_replace<'a>(s: &str, ctx: &ModuleFilenameTemplateCtx<'a>) -> String {
  let resource_tag = b"[resource]";
  let sstr = s;
  let s = s.as_bytes();
  let mut buf = String::new();
  let mut pos = 0;
  let mut state = false;
  let mut uses_virtual_resource = false;

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
            b"[resource]" => {
              uses_virtual_resource |= ctx.is_virtual_module();
              buf.push_str(&ctx.resource(true));
            },
            b"[resource-path]" | b"[resourcepath]" => {
              uses_virtual_resource |= ctx.is_virtual_module();
              buf.push_str(&ctx.resource(false));
            },

            b"[absolute-resource-path]" |
            b"[abs-resource-path]" |
            b"[absoluteresource-path]" |
            b"[absresource-path]" |
            b"[absolute-resourcepath]" |
            b"[abs-resourcepath]" |
            b"[absoluteresourcepath]" |
            b"[absresourcepath]" => {
              uses_virtual_resource |= ctx.is_virtual_module();
              buf.push_str(&ctx.absolute_resource_path());
            },

            b"[relative-resource-path]" |
            b"[relativeresource-path]" |
            b"[relative-resourcepath]" |
            b"[relativeresourcepath]" => {
              uses_virtual_resource |= ctx.is_virtual_module();
              if let Some(relative_resource_path) = ctx.relative_resource_path() {
                buf.push_str(&relative_resource_path);
              } else if ctx.module.is_some() {
                // Eval templates use the module request when there is no map filename.
                buf.push_str(&ctx.resource(true));
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

            b"[query]" => buf.push_str(&ctx.query()),
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
  if uses_virtual_resource && !starts_with_url_scheme(&buf) {
    ctx.virtual_source_url(&buf)
  } else {
    buf
  }
}
