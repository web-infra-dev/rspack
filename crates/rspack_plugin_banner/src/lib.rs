#![feature(let_chains)]

use std::fmt::{self, Debug};

use async_recursion::async_recursion;
use futures::future::BoxFuture;
use once_cell::sync::Lazy;
use regex::Regex;
use rspack_core::{
  rspack_sources::{BoxSource, ConcatSource, RawSource, SourceExt},
  to_comment, Chunk, Compilation, CompilationProcessAssets, FilenameTemplate, Logger, PathData,
  Plugin,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_regex::RspackRegex;
use rspack_util::{infallible::ResultInfallibleExt as _, try_any_sync};

#[derive(Debug)]
pub enum BannerRule {
  String(String),
  Regexp(RspackRegex),
}

#[derive(Debug)]
pub enum BannerRules {
  Single(BannerRule),
  Array(Vec<BannerRule>),
}

impl FromIterator<BannerRule> for BannerRules {
  fn from_iter<T: IntoIterator<Item = BannerRule>>(iter: T) -> Self {
    Self::Array(iter.into_iter().collect())
  }
}

impl BannerRule {
  pub fn try_match(&self, data: &str) -> Result<bool> {
    match self {
      Self::String(s) => Ok(data.starts_with(s)),
      Self::Regexp(r) => Ok(r.test(data)),
    }
  }
}

impl BannerRules {
  #[async_recursion]
  pub async fn try_match(&self, data: &str) -> Result<bool> {
    match self {
      Self::Single(s) => s.try_match(data),
      Self::Array(l) => try_any_sync(l, |i| i.try_match(data)),
    }
  }
}

#[derive(Debug)]
pub struct BannerPluginOptions {
  // Specifies the banner.
  pub banner: BannerContent,
  // If true, the banner will only be added to the entry chunks.
  pub entry_only: Option<bool>,
  // If true, banner will be placed at the end of the output.
  pub footer: Option<bool>,
  // If true, banner will not be wrapped in a comment.
  pub raw: Option<bool>,
  // Include all modules that pass test assertion.
  pub test: Option<BannerRules>,
  // Include all modules matching any of these conditions.
  pub include: Option<BannerRules>,
  // Exclude all modules matching any of these conditions.
  pub exclude: Option<BannerRules>,
}

pub struct BannerContentFnCtx<'a> {
  pub hash: &'a str,
  pub chunk: &'a Chunk,
  pub filename: &'a str,
}

pub type BannerContentFn =
  Box<dyn for<'a> Fn(BannerContentFnCtx<'a>) -> BoxFuture<'a, Result<String>> + Sync + Send>;

pub enum BannerContent {
  String(String),
  Fn(BannerContentFn),
}

impl fmt::Debug for BannerContent {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::String(arg0) => f.debug_tuple("String").field(arg0).finish(),
      Self::Fn(_) => f.debug_tuple("Fn").finish(),
    }
  }
}

#[async_recursion]
async fn match_object(obj: &BannerPluginOptions, str: &str) -> Result<bool> {
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

static TRIALING_WHITESPACE: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\s+\n").expect("invalid regexp"));

fn wrap_comment(str: &str) -> String {
  if !str.contains('\n') {
    return to_comment(str);
  }

  let result = str
    .replace("*/", "* /")
    .split('\n')
    .collect::<Vec<_>>()
    .join("\n * ");
  let result = TRIALING_WHITESPACE.replace_all(&result, "\n");
  let result = result.trim_end();

  format!("/*!\n * {}\n */", result)
}

#[plugin]
#[derive(Debug)]
pub struct BannerPlugin {
  config: BannerPluginOptions,
}

impl BannerPlugin {
  pub fn new(config: BannerPluginOptions) -> Self {
    Self::new_inner(config)
  }

  fn wrap_comment(&self, value: &str) -> String {
    if let Some(true) = self.config.raw {
      value.to_owned()
    } else {
      wrap_comment(value)
    }
  }

  fn update_source(&self, comment: String, old: BoxSource, footer: Option<bool>) -> BoxSource {
    let old_source = old.to_owned();

    if let Some(footer) = footer
      && footer
    {
      ConcatSource::new([
        old_source,
        RawSource::from("\n").boxed(),
        RawSource::from(comment).boxed(),
      ])
      .boxed()
    } else {
      ConcatSource::new([
        RawSource::from(comment).boxed(),
        RawSource::from("\n").boxed(),
        old_source,
      ])
      .boxed()
    }
  }
}

#[plugin_hook(CompilationProcessAssets for BannerPlugin, stage = Compilation::PROCESS_ASSETS_STAGE_ADDITIONS)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  let logger = compilation.get_logger("rspack.BannerPlugin");
  let start = logger.time("add banner");
  let mut updates = vec![];

  // filter file
  for chunk in compilation.chunk_by_ukey.values() {
    let can_be_initial = chunk.can_be_initial(&compilation.chunk_group_by_ukey);

    if let Some(entry_only) = self.config.entry_only
      && entry_only
      && !can_be_initial
    {
      continue;
    }

    for file in &chunk.files {
      let is_match = match_object(&self.config, file).await.unwrap_or(false);

      if !is_match {
        continue;
      }
      // add comment to the matched file
      let hash = compilation
        .hash
        .as_ref()
        .expect("should have compilation.hash in process_assets hook")
        .encoded()
        .to_owned();
      // todo: support placeholder, such as [fullhash]、[chunkhash]
      let banner = match &self.config.banner {
        BannerContent::String(content) => self.wrap_comment(content),
        BannerContent::Fn(func) => {
          let res = func(BannerContentFnCtx {
            hash: &hash,
            chunk,
            filename: file,
          })
          .await?;
          self.wrap_comment(&res)
        }
      };
      let comment = compilation
        .get_path(
          &FilenameTemplate::from(banner),
          PathData::default().chunk(chunk).hash(&hash).filename(file),
        )
        .always_ok();
      updates.push((file.clone(), comment));
    }
  }

  for (file, comment) in updates {
    let _res = compilation.update_asset(file.as_str(), |old, info| {
      let new = self.update_source(comment, old, self.config.footer);
      Ok((new, info))
    });
  }

  logger.time_end(start);

  Ok(())
}

impl Plugin for BannerPlugin {
  fn name(&self) -> &'static str {
    "rspack.BannerPlugin"
  }

  fn apply(
    &self,
    ctx: rspack_core::PluginContext<&mut rspack_core::ApplyContext>,
    _options: &mut rspack_core::CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .compilation_hooks
      .process_assets
      .tap(process_assets::new(self));
    Ok(())
  }
}
