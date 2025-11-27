use std::{
  borrow::Cow,
  collections::HashMap,
  env,
  hash::{DefaultHasher, Hash, Hasher},
  path::{Path, PathBuf},
};

use anyhow::{Context, anyhow};
use cow_utils::CowUtils;
use itertools::Itertools;
use rayon::prelude::*;
use rspack_core::{
  AssetInfo, Compilation, CompilationAsset, Filename, PathData,
  rspack_sources::{RawBufferSource, RawStringSource, SourceExt},
};
use rspack_error::{AnyhowResultToRspackResultExt, Result};
use rspack_paths::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use sugar_path::SugarPath;

use crate::{
  config::{HtmlChunkSortMode, HtmlInject, HtmlRspackPluginOptions, HtmlScriptLoading},
  sri::{add_sri, create_digest_from_asset},
  tag::HtmlPluginTag,
};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HtmlPluginAssets {
  pub public_path: String,
  pub js: Vec<String>,
  pub css: Vec<String>,
  pub favicon: Option<String>,
  // manifest: Option<String>,
  pub js_integrity: Option<Vec<Option<String>>>,
  pub css_integrity: Option<Vec<Option<String>>>,
}

impl HtmlPluginAssets {
  pub async fn create_assets<'a>(
    config: &HtmlRspackPluginOptions,
    compilation: &'a Compilation,
    public_path: &str,
    output_path: &Utf8PathBuf,
    html_file_name: &Filename,
  ) -> Result<(HtmlPluginAssets, HashMap<String, &'a CompilationAsset>)> {
    let mut assets: HtmlPluginAssets = HtmlPluginAssets::default();
    let mut asset_map = HashMap::new();
    assets.public_path = public_path.to_string();

    let sorted_entry_names: Vec<&String> =
      if matches!(config.chunks_sort_mode, HtmlChunkSortMode::Manual)
        && let Some(chunks) = &config.chunks
      {
        chunks
          .iter()
          .filter(|&name| compilation.entrypoints.contains_key(name))
          .collect()
      } else {
        compilation
          .entrypoints
          .keys()
          .filter(|&entry_name| {
            let mut included = true;
            if let Some(included_chunks) = &config.chunks {
              included = included_chunks.iter().any(|c| c.eq(entry_name));
            }
            if let Some(exclude_chunks) = &config.exclude_chunks {
              included = included && !exclude_chunks.iter().any(|c| c.eq(entry_name));
            }
            included
          })
          .collect()
      };

    let included_assets = sorted_entry_names
      .iter()
      .map(|entry_name| compilation.entrypoint_by_name(entry_name))
      .flat_map(|entry| entry.get_files(&compilation.chunk_by_ukey))
      .filter_map(|asset_name| {
        let asset = compilation.assets().get(&asset_name).expect("TODO:");
        if asset.info.hot_module_replacement.unwrap_or(false)
          || asset.info.development.unwrap_or(false)
        {
          None
        } else {
          Some((asset_name.clone(), asset))
        }
      })
      .collect::<Vec<_>>();

    for (asset_name, asset) in included_assets {
      if let Some(extension) =
        Path::new(asset_name.split("?").next().unwrap_or_default()).extension()
      {
        let mut asset_uri = format!("{}{}", assets.public_path, url_encode_path(&asset_name));
        if config.hash.unwrap_or_default()
          && let Some(hash) = compilation.get_hash()
        {
          asset_uri = append_hash(&asset_uri, hash);
        }
        let final_path = generate_posix_path(&asset_uri);
        if extension.eq_ignore_ascii_case("css") {
          if asset_map.insert(final_path.to_string(), asset).is_none() {
            assets.css.push(final_path.to_string());
          }
        } else if extension.eq_ignore_ascii_case("js") || extension.eq_ignore_ascii_case("mjs") {
          // keep the `if` to make the code more readable
          #[allow(clippy::collapsible_if)]
          if asset_map.insert(final_path.to_string(), asset).is_none() {
            assets.js.push(final_path.to_string());
          }
        }
      }
    }

    assets.favicon = if let Some(favicon) = &config.favicon {
      let favicon = PathBuf::from(favicon)
        .file_name()
        .expect("favicon should have file name")
        .to_string_lossy()
        .to_string();

      let favicon_relative_path = PathBuf::from(config.get_relative_path(compilation, &favicon));

      let mut favicon_path: PathBuf = PathBuf::from(
        config
          .get_public_path(
            compilation,
            favicon_relative_path.to_string_lossy().to_string().as_str(),
          )
          .await,
      );

      if favicon_path.to_str().unwrap_or_default().is_empty() {
        let fake_html_file_name = compilation
          .get_path(
            html_file_name,
            PathData::default().filename(output_path.as_str()),
          )
          .await?;
        let output_path = compilation.options.output.path.as_std_path();
        favicon_path = output_path
          .relative(output_path.join(fake_html_file_name).join(".."))
          .join(favicon_relative_path);
      } else {
        favicon_path.push(favicon_relative_path);
      }

      let mut favicon_link_path = favicon_path.to_string_lossy().to_string();

      if config.hash.unwrap_or_default()
        && let Some(hash) = compilation.get_hash()
      {
        favicon_link_path = append_hash(&favicon_link_path, hash);
      }

      Some(generate_posix_path(&favicon_link_path).into())
    } else {
      None
    };

    Ok((assets, asset_map))
  }
}

#[derive(Clone, Debug, Default)]
pub struct HtmlPluginAssetTags {
  pub scripts: Vec<HtmlPluginTag>,
  pub styles: Vec<HtmlPluginTag>,
  pub meta: Vec<HtmlPluginTag>,
}

impl HtmlPluginAssetTags {
  pub fn from_assets(
    config: &HtmlRspackPluginOptions,
    assets: &HtmlPluginAssets,
    asset_map: &HashMap<String, &CompilationAsset>,
  ) -> Self {
    let mut asset_tags = HtmlPluginAssetTags::default();

    // create script tags
    asset_tags.scripts.extend(
      assets
        .js
        .par_iter()
        .map(|x| HtmlPluginTag::create_script(x.as_str(), &config.script_loading))
        .collect::<Vec<_>>(),
    );

    // create style tags
    asset_tags.styles.extend(
      assets
        .css
        .par_iter()
        .map(|x| HtmlPluginTag::create_style(x.as_str()))
        .collect::<Vec<_>>(),
    );

    // create base tag
    if let Some(base) = &config.base
      && let Some(tag) = HtmlPluginTag::create_base(base)
    {
      asset_tags.meta.push(tag);
    }

    // create title tag
    if let Some(title) = &config.title {
      asset_tags.meta.push(HtmlPluginTag::create_title(title));
    }

    // create meta tags
    if let Some(meta) = &config.meta {
      asset_tags.meta.extend(HtmlPluginTag::create_meta(meta));
    }

    // create favicon tag
    if let Some(favicon) = &assets.favicon {
      asset_tags.meta.push(HtmlPluginTag::create_favicon(favicon));
    }

    // if some plugin changes assets in the same stage after this plugin
    // both the name and the integrity may be inaccurate
    if let Some(hash_func) = &config.sri {
      asset_tags
        .scripts
        .par_iter_mut()
        .filter_map(|tag| {
          if let Some(asset) = tag.asset.as_ref().and_then(|asset| asset_map.get(asset)) {
            asset.get_source().map(|s| (tag, s))
          } else {
            None
          }
        })
        .for_each(|(tag, asset)| {
          let sri_value = create_digest_from_asset(hash_func, asset);
          add_sri(tag, &sri_value);
        });
      asset_tags
        .styles
        .par_iter_mut()
        .filter_map(|tag| {
          if let Some(asset) = tag.asset.as_ref().and_then(|asset| asset_map.get(asset)) {
            asset.get_source().map(|s| (tag, s))
          } else {
            None
          }
        })
        .for_each(|(tag, asset)| {
          let sri_value = create_digest_from_asset(hash_func, asset);
          add_sri(tag, &sri_value);
        });
    }

    asset_tags
  }

  pub fn to_groups(
    config: &HtmlRspackPluginOptions,
    asset_tags: HtmlPluginAssetTags,
  ) -> (Vec<HtmlPluginTag>, Vec<HtmlPluginTag>) {
    let mut body_tags = vec![];
    let mut head_tags = vec![];

    head_tags.extend(asset_tags.meta);

    for tag in &asset_tags.scripts {
      match config.inject {
        HtmlInject::Head => head_tags.push(tag.to_owned()),
        HtmlInject::Body => body_tags.push(tag.to_owned()),
        HtmlInject::False => {
          if matches!(config.script_loading, HtmlScriptLoading::Blocking) {
            body_tags.push(tag.to_owned());
          } else {
            head_tags.push(tag.to_owned());
          }
        }
      }
    }

    head_tags.extend(asset_tags.styles);
    (head_tags, body_tags)
  }
}

pub fn append_hash(url: &str, hash: &str) -> String {
  format!(
    "{}{}{}",
    url,
    if url.contains("?") {
      "$$RSPACK_URL_AMP$$"
    } else {
      "?"
    },
    hash
  )
}

pub fn generate_posix_path(path: &str) -> Cow<'_, str> {
  if env::consts::OS == "windows" {
    path.cow_replace(&['/', '\\'] as &[char], "/")
  } else {
    path.into()
  }
}

fn url_encode_path(file_path: &str) -> String {
  let query_string_start = file_path.find('?');
  let url_path = if let Some(query_string_start) = query_string_start {
    &file_path[..query_string_start]
  } else {
    file_path
  };
  let query_string = if let Some(query_string_start) = query_string_start {
    &file_path[query_string_start..]
  } else {
    ""
  };

  format!(
    "{}{}",
    url_path
      .split('/')
      .map(|p| { urlencoding::encode(p) })
      .join("/"),
    // element.outerHTML will escape '&' so need to add a placeholder here
    query_string.cow_replace("&", "$$RSPACK_URL_AMP$$")
  )
}

pub async fn create_favicon_asset(
  favicon: &str,
  config: &HtmlRspackPluginOptions,
  compilation: &Compilation,
) -> Result<(String, CompilationAsset)> {
  let favicon_file_path = PathBuf::from(config.get_relative_path(compilation, favicon))
    .file_name()
    .expect("Should have favicon file name")
    .to_string_lossy()
    .to_string();

  let resolved_favicon = compilation.options.context.as_path().join(favicon);

  compilation
    .input_filesystem
    .read(&resolved_favicon)
    .await
    .map_err(|err| anyhow!(err))
    .context(format!(
      "HtmlRspackPlugin: could not load file `{}` from `{}`",
      favicon, &compilation.options.context
    ))
    .map(|content| {
      (
        favicon_file_path,
        CompilationAsset::from(RawBufferSource::from(content).boxed()),
      )
    })
    .to_rspack_result_from_anyhow()
}

pub async fn create_html_asset(
  output_file_name: &Filename,
  html: &str,
  template_file_name: &str,
  compilation: &Compilation,
) -> Result<(String, CompilationAsset)> {
  let hash = hash_for_source(html);

  let mut asset_info = AssetInfo::default();
  let output_path = compilation
    .get_path_with_info(
      output_file_name,
      PathData::default()
        .filename(template_file_name)
        .content_hash(&hash),
      &mut asset_info,
    )
    .await?;

  Ok((
    output_path,
    CompilationAsset::new(Some(RawStringSource::from(html).boxed()), asset_info),
  ))
}

fn hash_for_source(source: &str) -> String {
  let mut hasher = DefaultHasher::new();
  source.hash(&mut hasher);
  format!("{:016x}", hasher.finish())
}
