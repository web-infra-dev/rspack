use std::{
  borrow::Cow,
  hash::{Hash, Hasher},
};

use cow_utils::CowUtils;
use ere::compile_regex;
use rspack_core::{ChunkGraph, Compilation, OutputOptions, contextify};
use rspack_error::Result;
use rspack_hash::RspackHash;
use rustc_hash::FxHashMap as HashMap;

use crate::{ModuleFilenameTemplateFn, ModuleFilenameTemplateFnCtx, ModuleOrSource};

fn replace_bracket_tags<F>(s: &str, replacer: F) -> Cow<str>
where
  F: Fn(&str) -> Cow<str>,
{
  if !s.contains('[') {
    return Cow::Borrowed(s);
  }

  let mut result = String::new();
  let mut chars = s.chars().peekable();
  let mut needs_replacement = false;

  while let Some(ch) = chars.next() {
    if ch == '[' {
      // Start collecting the tag content
      let mut tag_content = String::new();
      let mut found_closing = false;

      // Look for the closing bracket and collect content
      while let Some(&next_ch) = chars.peek() {
        chars.next();
        if next_ch == ']' {
          found_closing = true;
          break;
        } else if next_ch.is_alphanumeric() || next_ch == '-' || next_ch == '_' {
          tag_content.push(next_ch);
        } else {
          // Invalid character in tag, not a valid tag
          tag_content.clear();
          break;
        }
      }

      if found_closing && !tag_content.is_empty() {
        // Found a valid tag
        if !needs_replacement {
          needs_replacement = true;
          result.reserve(s.len() + 50); // Reserve extra space for replacements
          // Copy the part before this tag
          let current_pos = s.len() - chars.as_str().len() - tag_content.len() - 2; // -2 for []
          result.push_str(&s[..current_pos]);
        }

        let replacement = replacer(&tag_content);
        result.push_str(&replacement);
      } else {
        // Not a valid tag, add the bracket and content as-is
        if needs_replacement {
          result.push('[');
          if !tag_content.is_empty() {
            result.push_str(&tag_content);
            if !found_closing {
              // If we didn't find closing bracket, we need to add remaining chars back
              while let Some(&remaining_ch) = chars.peek() {
                result.push(remaining_ch);
                chars.next();
              }
              break;
            }
          }
        }
      }
    } else {
      if needs_replacement {
        result.push(ch);
      }
    }
  }

  if needs_replacement {
    Cow::Owned(result)
  } else {
    Cow::Borrowed(s)
  }
}

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

    // Manual replacement for [all-loaders][resource] -> [identifier]
    let s = module_filename_template.replace("[all-loaders][resource]", "[identifier]")
      .replace("[allloaders][resource]", "[identifier]");
    
    // Manual replacement for [loaders][resource] -> [short-identifier]
    let s = s.replace("[loaders][resource]", "[short-identifier]");
    
    replace_bracket_tags(&s, |tag_content| {
      match tag_content.to_ascii_lowercase().as_str() {
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
        
        _ => {
          // Check for escaped brackets pattern [\tag\]
          if tag_content.starts_with('\\') && tag_content.ends_with('\\') && tag_content.len() > 2 {
            Cow::from(format!("[{}]", &tag_content[1..tag_content.len() - 1]))
          } else {
            // Keep unmatched patterns as-is
            Cow::from(format!("[{}]", tag_content))
          }
        }
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
