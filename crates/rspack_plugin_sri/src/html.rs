use std::sync::Arc;

use futures::future::join_all;
use once_cell::sync::Lazy;
use regex::Regex;
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rspack_hook::plugin_hook;
use rspack_paths::Utf8Path;
use rspack_plugin_html::{
  AlterAssetTagGroupsData, BeforeAssetTagGenerationData, HtmlPluginAlterAssetTagGroups,
  HtmlPluginBeforeAssetTagGeneration,
  tag::{HtmlPluginAttribute, HtmlPluginTag},
};
use rustc_hash::FxHashMap as HashMap;
use tokio::sync::RwLock;
use url::Url;

static HTTP_PROTOCOL_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^https?:").expect("Invalid regex"));

use crate::{
  SRICompilationContext, SubresourceIntegrityHashFunction, SubresourceIntegrityPlugin,
  SubresourceIntegrityPluginInner, config::ArcFs, integrity::compute_integrity,
  util::normalize_path,
};

async fn handle_html_plugin_assets(
  data: &mut BeforeAssetTagGenerationData,
  compilation_integrities: Arc<RwLock<HashMap<String, String>>>,
) -> Result<()> {
  let normalized_integrities = get_normalized_integrities(compilation_integrities.clone()).await;

  let js_integrity = join_all(data.assets.js.iter().map(|asset| {
    get_integrity_checksum_for_asset(
      asset,
      compilation_integrities.clone(),
      &normalized_integrities,
    )
  }))
  .await;

  let css_integrity = join_all(data.assets.css.iter().map(|asset| {
    get_integrity_checksum_for_asset(
      asset,
      compilation_integrities.clone(),
      &normalized_integrities,
    )
  }))
  .await;

  data.assets.js_integrity = Some(js_integrity);
  data.assets.css_integrity = Some(css_integrity);

  Ok(())
}

async fn handle_html_plugin_tags(
  data: &mut AlterAssetTagGroupsData,
  hash_func_names: &Vec<SubresourceIntegrityHashFunction>,
  integrities: Arc<RwLock<HashMap<String, String>>>,
  ctx: &SRICompilationContext,
) -> Result<()> {
  let normalized_integrities = get_normalized_integrities(integrities.clone()).await;

  process_tag_group(
    &mut data.head_tags,
    &data.public_path,
    hash_func_names,
    integrities.clone(),
    &normalized_integrities,
    ctx,
  )
  .await?;
  process_tag_group(
    &mut data.body_tags,
    &data.public_path,
    hash_func_names,
    integrities,
    &normalized_integrities,
    ctx,
  )
  .await?;

  Ok(())
}

async fn process_tag_group(
  tags: &mut [HtmlPluginTag],
  public_path: &str,
  hash_func_names: &Vec<SubresourceIntegrityHashFunction>,
  integrities: Arc<RwLock<HashMap<String, String>>>,
  normalized_integrities: &HashMap<String, String>,
  ctx: &SRICompilationContext,
) -> Result<()> {
  let tags_integrities = join_all(tags.iter().map(|tag| {
    process_tag(
      tag,
      public_path,
      integrities.clone(),
      normalized_integrities,
      hash_func_names,
      ctx,
    )
  }))
  .await
  .into_iter()
  .collect::<Result<Vec<_>>>()?;

  for (tag, integrity) in tags.iter_mut().zip(tags_integrities) {
    let Some(integrity) = integrity else {
      continue;
    };

    tag.attributes.push(HtmlPluginAttribute {
      attr_name: "integrity".to_string(),
      attr_value: Some(integrity),
    });

    if get_tag_attribute(tag, "crossorigin").is_none() {
      tag.attributes.push(HtmlPluginAttribute {
        attr_name: "crossorigin".to_string(),
        attr_value: Some("anonymous".to_string()),
      });
    }
  }

  Ok(())
}

// Get the `src` or `href` attribute of a tag if it is a script
// or link tag that needs SRI.
fn get_tag_src(tag: &HtmlPluginTag) -> Option<String> {
  // Handle script tags with src attribute
  if tag.tag_name == "script" {
    return get_tag_attribute(tag, "src");
  }

  // Handle link tags that need SRI
  if tag.tag_name == "link" {
    let href = get_tag_attribute(tag, "href")?;
    let rel = get_tag_attribute(tag, "rel")?;

    // Only process link tags that load actual resources
    let needs_sri = rel == "stylesheet"
      || rel == "modulepreload"
      || (rel == "preload" && {
        let as_attr = get_tag_attribute(tag, "as");
        as_attr.as_deref() == Some("script") || as_attr.as_deref() == Some("style")
      });

    if needs_sri {
      return Some(href);
    }
  }

  None
}

async fn process_tag(
  tag: &HtmlPluginTag,
  public_path: &str,
  integrities: Arc<RwLock<HashMap<String, String>>>,
  normalized_integrities: &HashMap<String, String>,
  hash_func_names: &Vec<SubresourceIntegrityHashFunction>,
  ctx: &SRICompilationContext,
) -> Result<Option<String>> {
  if tag.tag_name != "script" && tag.tag_name != "link" {
    return Ok(None);
  }

  if get_tag_attribute(tag, "integrity").is_some() {
    return Ok(None);
  }

  let Some(tag_src) = get_tag_src(tag) else {
    return Ok(None);
  };

  // Check if the tag_src is an absolute URL (http/https or protocol-relative)
  let is_absolute_url = match Url::parse(&tag_src) {
    Ok(url) => url.scheme() == "http" || url.scheme() == "https",
    Err(_) => tag_src.starts_with("//"),
  };

  // If it's an absolute URL, check if it's under publicPath
  let src = if is_absolute_url {
    // If publicPath is just "/" or empty or "./", it means local resources
    // External absolute URLs should be skipped
    let is_local_public_path = public_path.is_empty() || public_path == "/" || public_path == "./";

    if is_local_public_path {
      // Local publicPath, skip all external URLs
      return Ok(None);
    }

    let protocol_relative_public_path = HTTP_PROTOCOL_REGEX.replace(public_path, "").to_string();
    let protocol_relative_tag_src = HTTP_PROTOCOL_REGEX.replace(&tag_src, "").to_string();

    // If the tag src doesn't start with publicPath, it's an external resource
    // Skip SRI for external resources not served from our publicPath
    if !protocol_relative_tag_src.starts_with(&protocol_relative_public_path) {
      return Ok(None);
    }

    // Extract the asset path relative to publicPath
    let tag_src_with_scheme = format!("http:{protocol_relative_tag_src}");
    let public_path_with_scheme = if protocol_relative_public_path.starts_with("//") {
      format!("http:{protocol_relative_public_path}")
    } else {
      protocol_relative_public_path
    };
    get_asset_path(&tag_src_with_scheme, &public_path_with_scheme)
  } else {
    get_asset_path(&tag_src, public_path)
  };

  if let Some(integrity) =
    get_integrity_checksum_for_asset(&src, integrities, normalized_integrities).await
  {
    return Ok(Some(integrity));
  }

  let file_path = ctx.output_path.join(src);
  let integrity = compute_file_integrity(&file_path, &ctx.fs, hash_func_names).await?;
  Ok(Some(integrity))
}

fn get_tag_attribute(tag: &HtmlPluginTag, name: &str) -> Option<String> {
  tag
    .attributes
    .iter()
    .find(|attr| attr.attr_name == name)
    .and_then(|attr| attr.attr_value.as_ref())
    .cloned()
}

fn get_asset_path(src: &str, public_path: &str) -> String {
  let decoded_src = urlencoding::decode(src)
    .expect("Failed to decode asset path")
    .to_string();
  pathdiff::diff_paths(&decoded_src, public_path)
    .map_or_else(|| decoded_src.clone(), |p| p.to_string_lossy().into_owned())
}

async fn get_integrity_checksum_for_asset(
  src: &str,
  integrities: Arc<RwLock<HashMap<String, String>>>,
  normalized_integrities: &HashMap<String, String>,
) -> Option<String> {
  let integrities = integrities.read().await;
  if let Some(integrity) = integrities.get(src) {
    return Some(integrity.clone());
  }

  let normalized_src = normalize_path(src).into_owned();
  normalized_integrities.get(&normalized_src).cloned()
}

async fn get_normalized_integrities(
  integrities: Arc<RwLock<HashMap<String, String>>>,
) -> HashMap<String, String> {
  integrities
    .read()
    .await
    .iter()
    .map(|(key, value)| (normalize_path(key).into_owned(), value.clone()))
    .collect::<HashMap<_, _>>()
}

async fn compute_file_integrity(
  path: &Utf8Path,
  fs: &ArcFs,
  hash_func_names: &Vec<SubresourceIntegrityHashFunction>,
) -> Result<String> {
  let file = fs.read_file(path).await?;
  let content = String::from_utf8(file).to_rspack_result()?;
  let integrity = compute_integrity(hash_func_names, &content);
  Ok(integrity)
}

#[plugin_hook(HtmlPluginBeforeAssetTagGeneration for SubresourceIntegrityPlugin)]
pub async fn before_asset_tag_generation(
  &self,
  mut data: BeforeAssetTagGenerationData,
) -> Result<BeforeAssetTagGenerationData> {
  let compilation_integrities =
    SubresourceIntegrityPlugin::get_compilation_integrities_mut(data.compilation_id);
  handle_html_plugin_assets(&mut data, compilation_integrities).await?;
  Ok(data)
}

#[plugin_hook(HtmlPluginAlterAssetTagGroups for SubresourceIntegrityPlugin, stage = 10000)]
pub async fn alter_asset_tag_groups(
  &self,
  mut data: AlterAssetTagGroupsData,
) -> Result<AlterAssetTagGroupsData> {
  let compilation_integrities =
    SubresourceIntegrityPlugin::get_compilation_integrities(data.compilation_id);
  let ctx = SubresourceIntegrityPlugin::get_compilation_sri_context(data.compilation_id);
  handle_html_plugin_tags(
    &mut data,
    &self.options.hash_func_names,
    compilation_integrities,
    &ctx,
  )
  .await?;
  Ok(data)
}
