use std::{
  fmt::{self, Debug},
  sync::LazyLock,
};

use cow_utils::CowUtils;
use futures::future::BoxFuture;
use regex::Regex;
use rspack_core::{
  Chunk, Compilation, CompilationProcessAssets, Filename, Logger, PathData, Plugin,
  rspack_sources::{BoxSource, ConcatSource, RawStringSource, SourceExt},
  to_comment,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_util::asset_condition::AssetConditions;

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
  pub test: Option<AssetConditions>,
  // Include all modules matching any of these conditions.
  pub include: Option<AssetConditions>,
  // Exclude all modules matching any of these conditions.
  pub exclude: Option<AssetConditions>,
  // Specifies the stage of banner.
  pub stage: Option<i32>,
}

pub struct BannerContentFnCtx<'a> {
  pub hash: &'a str,
  pub chunk: &'a Chunk,
  pub filename: &'a str,
  pub compilation: &'a Compilation,
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

fn match_object(obj: &BannerPluginOptions, str: &str) -> bool {
  if let Some(condition) = &obj.test
    && !condition.try_match(str)
  {
    return false;
  }
  if let Some(condition) = &obj.include
    && !condition.try_match(str)
  {
    return false;
  }
  if let Some(condition) = &obj.exclude
    && condition.try_match(str)
  {
    return false;
  }
  true
}

static TRIALING_WHITESPACE: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"\s+\n").expect("invalid regexp"));

fn wrap_comment(str: &str) -> String {
  if !str.contains('\n') {
    return to_comment(str);
  }

  let result = str
    .cow_replace("*/", "* /")
    .split('\n')
    .collect::<Vec<_>>()
    .join("\n * ");
  let result = TRIALING_WHITESPACE.replace_all(&result, "\n");
  let result = result.trim_end();

  format!(
    r#"/*!
 * {result}
 */"#
  )
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
        RawStringSource::from_static("\n").boxed(),
        RawStringSource::from(comment).boxed(),
      ])
      .boxed()
    } else {
      ConcatSource::new([
        RawStringSource::from(comment).boxed(),
        RawStringSource::from_static("\n").boxed(),
        old_source,
      ])
      .boxed()
    }
  }
}

#[plugin_hook(CompilationProcessAssets for BannerPlugin, stage = self.config.stage.unwrap_or(Compilation::PROCESS_ASSETS_STAGE_ADDITIONS))]
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

    for file in chunk.files() {
      let is_match = match_object(&self.config, file);

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
            compilation,
          })
          .await?;
          self.wrap_comment(&res)
        }
      };
      let comment = compilation
        .get_path(
          &Filename::from(banner),
          PathData::default()
            .chunk_hash_optional(chunk.rendered_hash(
              &compilation.chunk_hashes_artifact,
              compilation.options.output.hash_digest_length,
            ))
            .chunk_id_optional(chunk.id().map(|id| id.as_str()))
            .chunk_name_optional(chunk.name_for_filename_template())
            .hash(&hash)
            .filename(file),
        )
        .await?;
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

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .compilation_hooks
      .process_assets
      .tap(process_assets::new(self));
    Ok(())
  }
}
