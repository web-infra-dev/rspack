use std::{
  borrow::Cow,
  hash::{Hash, Hasher},
};

use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use rspack_core::{contextify, Compilation, OutputOptions};
use rspack_error::Result;
use rspack_hash::RspackHash;
use rustc_hash::FxHashMap as HashMap;

use crate::{ModuleFilenameTemplateFn, ModuleFilenameTemplateFnCtx, ModuleOrSource};

static REGEXP_ALL_LOADERS_RESOURCE: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r"\[all-?loaders\]\[resource\]")
    .expect("failed to compile REGEXP_ALL_LOADERS_RESOURCE")
});
static SQUARE_BRACKET_TAG_REGEXP: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r"\[\\*([\w-]+)\\*\]").expect("failed to compile SQUARE_BRACKET_TAG_REGEXP")
});
static REGEXP_LOADERS_RESOURCE: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r"\[loaders\]\[resource\]").expect("failed to compile REGEXP_LOADERS_RESOURCE")
});

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
  format!("{:x}", hasher.finish())[..4].to_string()
}

pub struct ModuleFilenameHelpers;

impl ModuleFilenameHelpers {
  fn create_module_filename_template_fn_ctx(
    module_or_source: &ModuleOrSource,
    compilation: &Compilation,
    output_options: &OutputOptions,
    namespace: &str,
  ) -> ModuleFilenameTemplateFnCtx {
    let Compilation {
      chunk_graph,
      options,
      ..
    } = compilation;
    let context = &options.context;

    match module_or_source {
      ModuleOrSource::Module(module_identifier) => {
        let module_graph = compilation.get_module_graph();
        let module = module_graph
          .module_by_identifier(module_identifier)
          .unwrap_or_else(|| {
            panic!(
              "failed to find a module for the given identifier '{}'",
              module_identifier
            )
          });

        let short_identifier = module.readable_identifier(context).to_string();
        let identifier = contextify(context, module_identifier);
        let module_id = chunk_graph
          .get_module_id(*module_identifier)
          .clone()
          .unwrap_or("".to_string());
        let absolute_resource_path = "".to_string();

        let hash = get_hash(&identifier, output_options);

        let resource = short_identifier
          .clone()
          .split('!')
          .last()
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
          .last()
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
          absolute_resource_path: source.split('!').last().unwrap_or("").to_string(),
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

    let s = REGEXP_ALL_LOADERS_RESOURCE.replace_all(module_filename_template, "[identifier]");
    let s = REGEXP_LOADERS_RESOURCE.replace_all(&s, "[short-identifier]");
    SQUARE_BRACKET_TAG_REGEXP
      .replace_all(&s, |caps: &Captures| {
        let full_match = caps
          .get(0)
          .expect("the SQUARE_BRACKET_TAG_REGEXP must match the whole tag, but it did not match anything.")
          .as_str();
        let content = caps
          .get(1)
          .expect("the SQUARE_BRACKET_TAG_REGEXP must match the whole tag, but it did not match anything.")
          .as_str();

        if content.len() + 2 == full_match.len() {
          match content.to_lowercase().as_str() {
            "identifier" => Cow::from(&ctx.identifier),
            "short-identifier" => Cow::from(&ctx.short_identifier),
            "resource" => Cow::from(&ctx.resource),

            "resource-path" |  "resourcepath" => Cow::from(&ctx.resource_path),

            "absolute-resource-path" |
            "abs-resource-path" |
            "absoluteresource-path" |
            "absresource-path" |
            "absolute-resourcepath" |
            "abs-resourcepath" |
            "absoluteresourcepath" |
            "absresourcepath" => Cow::from(&ctx.absolute_resource_path),

            "all-loaders" | "allloaders" => Cow::from(&ctx.all_loaders),
            "loaders" => Cow::from(&ctx.loaders),

            "query" => Cow::from(&ctx.query),
            "id" => Cow::from(&ctx.module_id),
            "hash" => Cow::from(&ctx.hash),
            "namespace" => Cow::from(&ctx.namespace),

            _ => Cow::from(full_match.to_string())
          }
        } else if full_match.starts_with("[\\") && full_match.ends_with("\\]") {
          Cow::from(format!("[{}]", &full_match[2..full_match.len() - 2]))
        } else {
          Cow::from(full_match.to_string())
        }
      })
      .to_string()
  }

  pub fn replace_duplicates<F>(filenames: Vec<String>, mut fn_replace: F) -> Vec<String>
  where
    F: FnMut(String, usize, usize) -> String,
  {
    let mut count_map: HashMap<String, Vec<usize>> = HashMap::default();
    let mut pos_map: HashMap<String, usize> = HashMap::default();

    for (idx, item) in filenames.iter().enumerate() {
      count_map
        .entry(item.clone())
        .or_insert_with(Vec::new)
        .push(idx);
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
