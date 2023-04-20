#![feature(let_chains)]

use std::collections::HashMap;
use std::fmt::{self, Debug};

use async_recursion::async_recursion;
use async_trait::async_trait;
use rspack_core::{
  rspack_sources::{BoxSource, ConcatSource, RawSource, SourceExt},
  to_comment, try_any, Plugin,
};
use rspack_error::Result;
use rspack_regex::RspackRegex;

pub enum BannerCondition {
  String(String),
  Regexp(RspackRegex),
}

pub enum BannerConditions {
  String(String),
  Regexp(RspackRegex),
  Array(Vec<BannerCondition>),
}

impl BannerCondition {
  #[async_recursion]
  pub async fn try_match(&self, data: &str) -> Result<bool> {
    match self {
      Self::String(s) => Ok(data.starts_with(s)),
      Self::Regexp(r) => Ok(r.test(data)),
    }
  }
}

impl BannerConditions {
  #[async_recursion]
  pub async fn try_match(&self, data: &str) -> Result<bool> {
    match self {
      Self::String(s) => Ok(data.starts_with(s)),
      Self::Regexp(r) => Ok(r.test(data)),
      Self::Array(l) => try_any(l, |i| async { i.try_match(data).await }).await,
    }
  }
}

#[derive(Debug)]
pub struct BannerConfig {
  /**
   * Specifies the banner.
   */
  pub banner: String,
  /**
   * If true, the banner will only be added to the entry chunks.
   */
  pub entry_only: Option<bool>,
  /**
   * If true, banner will be placed at the end of the output.
   */
  pub footer: Option<bool>,
  /**
   * If true, banner will not be wrapped in a comment.
   */
  pub raw: Option<bool>,

  /**
   * Include all modules that pass test assertion.
   */
  pub test: Option<BannerConditions>,
  /**
   * Include all modules matching any of these conditions.
   */
  pub include: Option<BannerConditions>,
  /**
   * Exclude all modules matching any of these conditions.
   */
  pub exclude: Option<BannerConditions>,
}

impl fmt::Debug for BannerCondition {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::String(i) => i.fmt(f),
      Self::Regexp(i) => i.fmt(f),
    }
  }
}

impl fmt::Debug for BannerConditions {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::String(i) => i.fmt(f),
      Self::Regexp(i) => i.fmt(f),
      Self::Array(i) => i.fmt(f),
    }
  }
}

#[async_recursion]
async fn match_object(obj: &BannerConfig, str: &str) -> Result<bool> {
  if let Some(condition) = &obj.test {
    if !condition.try_match(str).await? {
      return Ok(false);
    }
  }
  if let Some(condition) = &obj.include {
    if !condition.try_match(str).await? {
      return Ok(false);
    }
  }
  if let Some(condition) = &obj.exclude {
    if condition.try_match(str).await? {
      return Ok(false);
    }
  }
  Ok(true)
}

fn wrap_comment(str: &str) -> String {
  if !str.contains('\n') {
    return to_comment(str);
  }

  let binding = str
    .replace("* /", "*/")
    .split('\n')
    .collect::<Vec<_>>()
    .join("\n * ")
    .replace(|c: char| c.is_whitespace() && c != '\n', " ");
  let result = binding.trim_end();

  format!("/*!\n * {}\n */", result)
}

#[derive(Debug)]
pub struct CachedStruct {
  comment: String,
}

#[derive(Debug)]
pub struct BannerPlugin {
  config: BannerConfig,
  comment: String,
  cache: HashMap<String, CachedStruct>,
}

impl BannerPlugin {
  pub fn new(config: BannerConfig) -> Self {
    let comment = if let Some(raw) = config.raw && raw {
      config.banner.clone()
    } else {
      wrap_comment(&config.banner)
    };

    Self {
      config,
      comment,
      cache: HashMap::new(),
    }
  }

  fn update_cache_source(&mut self, comment: String, old: &mut BoxSource, footer: Option<bool>) {
    let old_source = old.source().to_string();
    let source;

    if let Some(footer) = footer && footer {
      source = ConcatSource::new([
        RawSource::from(old_source.clone()),
        RawSource::from("\n"),
        RawSource::from(comment.to_string()),
      ]);
    } else {
      source = ConcatSource::new([
        RawSource::from(comment.to_string()),
        RawSource::from("\n"),
        RawSource::from(old_source.clone()),
      ]);
    }

    *old = source.boxed();

    self.cache.insert(old_source, CachedStruct { comment });
  }
}

#[async_trait]
impl Plugin for BannerPlugin {
  fn name(&self) -> &'static str {
    "banner-rspack-plugin"
  }

  async fn process_assets_stage_additions(
    &mut self,
    _ctx: rspack_core::PluginContext,
    args: rspack_core::ProcessAssetsArgs<'_>,
  ) -> rspack_core::PluginProcessAssetsOutput {
    let compilation = args.compilation;
    let mut chunk_files = vec![];

    // filter file
    for chunk in compilation.chunk_by_ukey.values() {
      let can_be_initial = chunk.can_be_initial(&compilation.chunk_group_by_ukey);

      if let Some(entry_only) = self.config.entry_only && entry_only && !can_be_initial {
        continue;
      }

      for file in &chunk.files {
        let is_match = match_object(&self.config, file).await.unwrap_or(false);

        if !is_match {
          continue;
        }
        chunk_files.push(file.clone());
      }
    }

    // add comment to the matched file
    for file in chunk_files {
      // todo: support placeholder, such as [fullhash]、[chunkhash]
      let comment = self.comment.to_owned();
      let _res = compilation.update_asset(file.as_str(), |old, _info| {
        let old_source = old.source().to_string();
        let cached = self.cache.get(&old_source);
        // check cache
        if let Some(cache_info) = cached {
          if cache_info.comment != comment {
            self.update_cache_source(comment, old, self.config.footer);
          }
        } else {
          self.update_cache_source(comment, old, self.config.footer);
        }
        Ok(())
      });
    }

    Ok(())
  }
}
