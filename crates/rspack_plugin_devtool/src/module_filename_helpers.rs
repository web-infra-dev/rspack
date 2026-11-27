use std::{borrow::Cow, cell::OnceCell, path::Path, sync::LazyLock};

use regex::Regex;
use rspack_core::{ChunkGraph, Compilation, OutputOptions, contextify, runtime_mode::RuntimeMode};
use rspack_error::Result;
use rspack_hash::{RspackHash, RspackHasher};
use rspack_paths::Utf8Path;
use rspack_util::{
  identifier::{split_at_query_mark, split_resource},
  node_path::normalize_path_to_posix,
};
use rustc_hash::FxHashMap as HashMap;
use sugar_path::SugarPath;

use crate::{
  ModuleFilenameTemplateFn, ModuleFilenameTemplateFnCtx, SourceReference,
  source_map_dev_tool_plugin::{starts_with_url_scheme, starts_with_windows_drive_letter},
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

fn format_resource_path<'a>(path: &'a str, relative_to: Option<&Utf8Path>) -> Cow<'a, str> {
  let windows_drive = starts_with_windows_drive_letter(path);
  if path.is_empty()
    || path.starts_with("//")
    || path.starts_with(r"\\?\")
    || path.starts_with(r"\\.\")
    || (!windows_drive && starts_with_url_scheme(path))
  {
    return Cow::Borrowed(path);
  }

  let is_unc_path = path.starts_with(r"\\");
  let relative = relative_to
    .filter(|_| cfg!(windows) || (!windows_drive && !is_unc_path))
    .map(|base| Path::new(path).relative(base));
  let path = relative
    .as_deref()
    .map(Path::to_string_lossy)
    .unwrap_or(Cow::Borrowed(path));
  // A path can remain absolute when it is on a different Windows share.
  let normalized = if path.starts_with("//") || path.starts_with(r"\\") {
    path.into_owned()
  } else {
    normalize_path_to_posix(Utf8Path::new(path.as_ref())).into_string()
  };
  Cow::Owned(normalized)
}

fn format_resource_request<'a>(request: &'a str, relative_to: Option<&Utf8Path>) -> Cow<'a, str> {
  let (path, _, _) = split_resource(request);
  let Cow::Owned(mut normalized) = format_resource_path(path, relative_to) else {
    return Cow::Borrowed(request);
  };
  normalized.push_str(&request[path.len()..]);
  Cow::Owned(normalized)
}

fn resolve_relative_resource_path<'a>(
  absolute_resource_path: &str,
  source_map_path: Option<&Utf8Path>,
  runtime_mode: RuntimeMode,
  format: impl FnOnce(Option<&Utf8Path>) -> Cow<'a, str>,
) -> Option<Cow<'a, str>> {
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
    return Some(Cow::Owned(format!("{scheme}://{absolute_resource_path}")));
  }

  source_map_path.map(|path| format(path.parent()))
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
        let resolved_path = module
          .as_normal_module()
          .and_then(|module| module.resource_resolved_data().path());
        let absolute_resource_path = resolved_path.map_or_else(
          || module_identifier.rsplit('!').next().unwrap_or_default(),
          Utf8Path::as_str,
        );

        let hash = get_hash(&identifier, output_options);

        let resource = resource_identifier
          .split('!')
          .next_back()
          .unwrap_or("")
          .to_string();
        let relative_resource_path = resolve_relative_resource_path(
          absolute_resource_path,
          unresolved_source_map_path,
          options.experiments.runtime_mode,
          |base| {
            if resolved_path.is_some() {
              format_resource_path(absolute_resource_path, base)
            } else {
              Cow::Borrowed(resource.as_str())
            }
          },
        )
        .map(Cow::into_owned);

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
          absolute_resource_path: if resolved_path.is_some() {
            format_resource_path(absolute_resource_path, None).into_owned()
          } else {
            absolute_resource_path.to_owned()
          },
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

        let absolute_resource_path = source.split('!').next_back().unwrap_or("");
        let relative_resource_path = resolve_relative_resource_path(
          absolute_resource_path,
          unresolved_source_map_path,
          options.experiments.runtime_mode,
          |base| format_resource_request(absolute_resource_path, base),
        )
        .map(Cow::into_owned);

        ModuleFilenameTemplateFnCtx {
          short_identifier,
          identifier,
          module_id: String::new(),
          absolute_resource_path: format_resource_request(absolute_resource_path, None)
            .into_owned(),
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
  result.push_str(resource_path);
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

  fn absolute_resource_request(&self) -> (&str, bool) {
    match &self.source_reference {
      SourceReference::Module(module_identifier) => {
        let module_graph = self.compilation.get_module_graph();
        let module = module_graph
          .module_by_identifier(module_identifier)
          .expect("source reference should resolve to a module");
        if let Some(resource) = module
          .as_normal_module()
          .map(|module| module.resource_resolved_data())
          .filter(|resource| resource.path().is_some())
        {
          return (resource.resource(), true);
        }
        (
          module_identifier.rsplit('!').next().unwrap_or_default(),
          false,
        )
      }
      SourceReference::Source(source) => (source.rsplit('!').next().unwrap_or_default(), true),
    }
  }

  pub fn absolute_resource_path(&self) -> Cow<'_, str> {
    let (request, is_file) = self.absolute_resource_request();
    if is_file {
      format_resource_request(request, None)
    } else {
      Cow::Borrowed(request)
    }
  }

  pub fn relative_resource_path(&self) -> Option<Cow<'_, str>> {
    let (request, is_file) = self.absolute_resource_request();
    let resolved = resolve_relative_resource_path(
      request,
      self.unresolved_source_map_path,
      self.compilation.options.experiments.runtime_mode,
      |base| {
        if is_file {
          format_resource_request(request, base)
        } else {
          Cow::Borrowed(self.resource())
        }
      },
    );

    match (resolved, &self.source_reference) {
      (Some(path), _) => Some(path),
      (None, SourceReference::Module(_)) => {
        // Eval devtool modes have no source map file path to resolve against.
        let resource = self.resource();
        Some(if is_file {
          format_resource_request(resource, None)
        } else {
          Cow::Borrowed(resource)
        })
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
            b"[identifier]" => buf.push_str(ctx.identifier().as_ref()),
            b"[short-identifier]" => buf.push_str(ctx.short_identifier().as_ref()),
            b"[resource]" => buf.push_str(ctx.resource()),
            b"[resource-path]" |  b"[resourcepath]" => buf.push_str(ctx.resource_path()),

            b"[absolute-resource-path]" |
            b"[abs-resource-path]" |
            b"[absoluteresource-path]" |
            b"[absresource-path]" |
            b"[absolute-resourcepath]" |
            b"[abs-resourcepath]" |
            b"[absoluteresourcepath]" |
            b"[absresourcepath]" => buf.push_str(&ctx.absolute_resource_path()),

            b"[relative-resource-path]" |
            b"[relativeresource-path]" |
            b"[relative-resourcepath]" |
            b"[relativeresourcepath]" => {
              if let Some(relative_resource_path) = ctx.relative_resource_path() {
                buf.push_str(relative_resource_path.as_ref())
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
