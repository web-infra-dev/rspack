use std::hash::{Hash, Hasher};

use rspack_core::{ChunkGraph, Compilation, OutputOptions, contextify};
use rspack_error::Result;
use rspack_hash::RspackHash;
use rustc_hash::FxHashMap as HashMap;

use crate::{ModuleFilenameTemplateFn, ModuleFilenameTemplateFnCtx, ModuleOrSource};

fn get_before(s: &str, token: &str) -> String {
  match s.rfind(token) {
    Some(idx) => s[..idx].to_string(),
    None => "".to_string(),
  }
}

fn get_after(s: &str, token: &str) -> String {
  s.find(token)
    .map(|idx| s[idx..].to_string())
    .unwrap_or("".to_string())
}

fn get_hash(text: &str, output_options: &OutputOptions) -> String {
  let OutputOptions {
    hash_function,
    hash_salt,
    ..
  } = output_options;
  let mut hasher = RspackHash::with_salt(hash_function, hash_salt);
  text.as_bytes().hash(&mut hasher);
  let mut buf = format!("{:x}", hasher.finish());
  buf.truncate(4);
  buf
}

pub struct ModuleFilenameHelpers;

impl ModuleFilenameHelpers {
  fn create_module_filename_template_fn_ctx(
    module_or_source: &ModuleOrSource,
    compilation: &Compilation,
    output_options: &OutputOptions,
    namespace: &str,
  ) -> ModuleFilenameTemplateFnCtx {
    let Compilation { options, .. } = compilation;
    let context = &options.context;

    match module_or_source {
      ModuleOrSource::Module(module_identifier) => {
        let module_graph = compilation.get_module_graph();
        let module = module_graph
          .module_by_identifier(module_identifier)
          .unwrap_or_else(|| {
            panic!("failed to find a module for the given identifier '{module_identifier}'")
          });

        let short_identifier = module.readable_identifier(context).to_string();
        let identifier = contextify(context, module_identifier);
        let module_id =
          ChunkGraph::get_module_id(&compilation.module_ids_artifact, *module_identifier)
            .map(|s| s.to_string())
            .unwrap_or_default();
        let absolute_resource_path = "".to_string();

        let hash = get_hash(&identifier, output_options);

        let resource = short_identifier
          .split('!')
          .next_back()
          .unwrap_or("")
          .to_string();

        let loaders = get_before(&short_identifier, "!");
        let all_loaders = get_before(&identifier, "!");
        let query = get_after(&resource, "?");

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
          hash,
          resource,
          loaders,
          all_loaders,
          query,
          resource_path,
          namespace: namespace.to_string(),
        }
      }
      ModuleOrSource::Source(source) => {
        let short_identifier = contextify(context, source);
        let identifier = short_identifier.clone();

        let hash = get_hash(&identifier, output_options);

        let resource = short_identifier
          .clone()
          .split('!')
          .next_back()
          .unwrap_or("")
          .to_string();

        let loaders = get_before(&short_identifier, "!");
        let all_loaders = get_before(&identifier, "!");
        let query = get_after(&resource, "?");

        let q = query.len();
        let resource_path = if q == 0 {
          resource.clone()
        } else {
          resource[..resource.len().saturating_sub(q)].to_string()
        };

        ModuleFilenameTemplateFnCtx {
          short_identifier,
          identifier,
          module_id: "".to_string(),
          absolute_resource_path: source.split('!').next_back().unwrap_or("").to_string(),
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
    module_or_source: &ModuleOrSource,
    compilation: &Compilation,
    module_filename_template: &ModuleFilenameTemplateFn,
    output_options: &OutputOptions,
    namespace: &str,
  ) -> Result<String> {
    let ctx = ModuleFilenameHelpers::create_module_filename_template_fn_ctx(
      module_or_source,
      compilation,
      output_options,
      namespace,
    );

    module_filename_template(ctx).await
  }

  pub fn create_filename_of_string_template(
    module_or_source: &ModuleOrSource,
    compilation: &Compilation,
    module_filename_template: &str,
    output_options: &OutputOptions,
    namespace: &str,
  ) -> String {
    let ctx = ModuleFilenameHelpers::create_module_filename_template_fn_ctx(
      module_or_source,
      compilation,
      output_options,
      namespace,
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
}

fn starts_with_ignore_ascii_case(s: &[u8], prefix: &[u8]) -> bool {
  s.len() >= prefix.len() && s[..prefix.len()].eq_ignore_ascii_case(prefix)
}

fn template_replace(s: &str, ctx: &ModuleFilenameTemplateFnCtx) -> String {
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
            b"[identifier]" => buf.push_str(&ctx.identifier),
            b"[short-identifier]" => buf.push_str(&ctx.short_identifier),
            b"[resource]" => buf.push_str(&ctx.resource),
            b"[resource-path]" |  b"[resourcepath]" => buf.push_str(&ctx.resource_path),

            b"[absolute-resource-path]" |
            b"[abs-resource-path]" |
            b"[absoluteresource-path]" |
            b"[absresource-path]" |
            b"[absolute-resourcepath]" |
            b"[abs-resourcepath]" |
            b"[absoluteresourcepath]" |
            b"[absresourcepath]" => buf.push_str(&ctx.absolute_resource_path),

            b"[all-loaders]" | b"[allloaders]" => if starts_with_ignore_ascii_case(&s[next_pos..], resource_tag) {
                next_pos += resource_tag.len();
                buf.push_str(&ctx.identifier);
            } else {
                buf.push_str(&ctx.all_loaders);
            },
            b"[loaders]" => if starts_with_ignore_ascii_case(&s[next_pos..], resource_tag) {
                next_pos += resource_tag.len();
                buf.push_str(&ctx.short_identifier);
            } else {
                buf.push_str(&ctx.loaders);
            },

            b"[query]" => buf.push_str(&ctx.query),
            b"[id]" => buf.push_str(&ctx.module_id),
            b"[hash]" => buf.push_str(&ctx.hash),
            b"[namespace]" => buf.push_str(&ctx.namespace),

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
