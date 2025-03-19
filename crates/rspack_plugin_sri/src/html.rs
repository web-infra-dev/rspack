use futures::future::join_all;
use rspack_error::Result;
use rspack_hook::plugin_hook;
use rspack_paths::Utf8Path;
use rspack_plugin_html::{
  tag::{HtmlPluginAttribute, HtmlPluginTag},
  AlterAssetTagGroupsData, BeforeAssetTagGenerationData, HtmlPluginAlterAssetTagGroups,
  HtmlPluginBeforeAssetTagGeneration,
};
use rustc_hash::FxHashMap as HashMap;

use crate::{
  config::ArcFs, integrity::compute_integrity, util::normalize_path, SRICompilationContext,
  SubresourceIntegrityHashFunction, SubresourceIntegrityPlugin, SubresourceIntegrityPluginInner,
};

async fn handle_html_plugin_assets(
  data: &mut BeforeAssetTagGenerationData,
  compilation_integrities: &mut HashMap<String, String>,
) -> Result<()> {
  let normalized_integrities = get_normalized_integrities(compilation_integrities);

  let js_integrity = data
    .assets
    .js
    .iter()
    .map(|asset| {
      get_integrity_chechsum_for_asset(asset, compilation_integrities, &normalized_integrities)
    })
    .collect::<Vec<_>>();

  let css_integrity = data
    .assets
    .css
    .iter()
    .map(|asset| {
      get_integrity_chechsum_for_asset(asset, compilation_integrities, &normalized_integrities)
    })
    .collect::<Vec<_>>();

  data.assets.js_integrity = Some(js_integrity);
  data.assets.css_integrity = Some(css_integrity);

  Ok(())
}

async fn handle_html_plugin_tags(
  data: &mut AlterAssetTagGroupsData,
  hash_func_names: &Vec<SubresourceIntegrityHashFunction>,
  integrities: &HashMap<String, String>,
  ctx: &SRICompilationContext,
) -> Result<()> {
  let normalized_integrities = get_normalized_integrities(integrities);

  process_tag_group(
    &mut data.head_tags,
    &data.public_path,
    hash_func_names,
    integrities,
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
  integrities: &HashMap<String, String>,
  normalized_integrities: &HashMap<String, String>,
  ctx: &SRICompilationContext,
) -> Result<()> {
  let tags_integrities = join_all(tags.iter().map(|tag| {
    process_tag(
      tag,
      public_path,
      integrities,
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

async fn process_tag(
  tag: &HtmlPluginTag,
  public_path: &str,
  integrities: &HashMap<String, String>,
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

  let Some(tag_src) = get_tag_attribute(tag, "href").or(get_tag_attribute(tag, "src")) else {
    return Ok(None);
  };

  let src = get_asset_path(&tag_src, public_path);
  if let Some(integrity) =
    get_integrity_chechsum_for_asset(&src, integrities, normalized_integrities)
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
    .map(|p| p.to_string_lossy().into_owned())
    .unwrap_or_else(|| decoded_src.to_string())
}

fn get_integrity_chechsum_for_asset(
  src: &str,
  integrities: &HashMap<String, String>,
  normalized_integrities: &HashMap<String, String>,
) -> Option<String> {
  if let Some(integrity) = integrities.get(src) {
    return Some(integrity.clone());
  }

  let normalized_src = normalize_path(src).into_owned();
  normalized_integrities.get(&normalized_src).cloned()
}

fn get_normalized_integrities(integrities: &HashMap<String, String>) -> HashMap<String, String> {
  integrities
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
  let content = String::from_utf8(file).map_err(|e| rspack_error::error!(e.to_string()))?;
  let integrity = compute_integrity(hash_func_names, &content);
  Ok(integrity)
}

#[plugin_hook(HtmlPluginBeforeAssetTagGeneration for SubresourceIntegrityPlugin)]
pub async fn before_asset_tag_generation(
  &self,
  mut data: BeforeAssetTagGenerationData,
) -> Result<BeforeAssetTagGenerationData> {
  let mut compilation_integrities =
    SubresourceIntegrityPlugin::get_compilation_integrities_mut(data.compilation_id);
  handle_html_plugin_assets(&mut data, &mut compilation_integrities).await?;
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
    &compilation_integrities,
    &ctx,
  )
  .await?;
  Ok(data)
}
