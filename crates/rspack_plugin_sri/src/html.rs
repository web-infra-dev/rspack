use std::{collections::BTreeMap, sync::Arc};

use futures::future::join_all;
use once_cell::sync::Lazy;
use regex::Regex;
use rspack_core::{
  ChunkGroup, ChunkGroupByUkey, ChunkGroupUkey, ChunkLoading, ChunkLoadingType, ChunkUkey,
  Compilation, ManifestAssetType,
};
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rspack_hook::plugin_hook;
use rspack_paths::Utf8Path;
use rspack_plugin_html::{
  AlterAssetTagGroupsData, BeforeAssetTagGenerationData, HtmlPluginAlterAssetTagGroups,
  HtmlPluginBeforeAssetTagGeneration, HtmlPluginCollectImportMapIntegrity,
  tag::{HtmlPluginAttribute, HtmlPluginTag},
};
use rspack_plugin_runtime::is_enabled_for_chunk;
use rustc_hash::{FxHashMap as HashMap, FxHashSet};
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

fn collect_import_map_chunks(
  entry: &ChunkGroup,
  chunk_groups: &ChunkGroupByUkey,
  include_async: bool,
) -> FxHashSet<ChunkUkey> {
  let mut pending_groups = vec![entry.ukey];
  // dependOn entries are ancestors of the page's entrypoint.
  pending_groups.extend(
    entry
      .ancestors(chunk_groups)
      .into_iter()
      .filter(|parent| chunk_groups.expect_get(parent).is_initial()),
  );

  let mut visited_groups = FxHashSet::default();
  let mut chunks = FxHashSet::default();
  while let Some(group) = pending_groups.pop() {
    if !visited_groups.insert(group) {
      continue;
    }
    let group = chunk_groups.expect_get(&group);
    chunks.extend(group.chunks.iter().copied());
    if include_async {
      // An initial child can be another page that depends on the same entry.
      pending_groups.extend(
        group
          .children_iterable()
          .filter(|child| !chunk_groups.expect_get(child).is_initial())
          .copied(),
      );
    }
  }
  chunks
}

#[plugin_hook(HtmlPluginCollectImportMapIntegrity for SubresourceIntegrityPlugin)]
pub async fn collect_import_map_integrity(
  &self,
  compilation: &Compilation,
  entrypoints: &[ChunkGroupUkey],
  public_path: &str,
  integrity: &mut BTreeMap<String, String>,
) -> Result<()> {
  // The configured function name only affects dynamic imports.
  let include_async = compilation.options.output.import_function_name == "import";
  let graph = &compilation.build_chunk_graph_artifact;
  let import_loading = ChunkLoading::Enable(ChunkLoadingType::Import);
  let mut imported_files = std::collections::BTreeSet::new();
  for entrypoint in entrypoints {
    let entry = graph.chunk_group_by_ukey.expect_get(entrypoint);
    let runtime_chunk = entry.get_runtime_chunk(&graph.chunk_group_by_ukey);
    if !is_enabled_for_chunk(&runtime_chunk, &import_loading, compilation) {
      continue;
    }

    let mut chunks = collect_import_map_chunks(entry, &graph.chunk_group_by_ukey, include_async);
    chunks.insert(runtime_chunk);
    let files = chunks
      .iter()
      .flat_map(|chunk| graph.chunk_by_ukey.expect_get(chunk).files())
      .filter(|file| {
        compilation
          .assets()
          .get(*file)
          .is_some_and(|asset| asset.get_info().asset_type == ManifestAssetType::JavaScript)
      })
      .collect::<Vec<_>>();
    // A standalone entry script already has an integrity attribute in the HTML.
    if files.len() > 1 {
      imported_files.extend(files);
    }
  }

  if imported_files.is_empty() {
    return Ok(());
  }

  let integrities = SubresourceIntegrityPlugin::get_compilation_integrities(compilation.id());
  let integrities = integrities.read().await;
  for file in imported_files {
    if let Some(hash) = integrities.get(file) {
      let mut url = format!("{public_path}{file}");
      if !url.starts_with('/')
        && !url.starts_with("./")
        && !url.starts_with("../")
        && Url::parse(&url).is_err()
      {
        url.insert_str(0, "./");
      }

      // Module tags encode their asset paths. A relative static import can inherit
      // the encoded part of its parent's directory and use a raw path for the rest.
      let pathname = file.split_once('?').map_or(file.as_str(), |(path, _)| path);
      let prefix = &url[..url.len() - file.len()];
      let mut encoded_dir = String::new();
      let mut start = 0;
      for (end, _) in pathname.match_indices('/') {
        let segment = &file[start..end];
        let encoded = urlencoding::encode(segment);
        let changed = encoded != segment;
        if changed && encoded_dir.is_empty() {
          encoded_dir.push_str(prefix);
          encoded_dir.push_str(&file[..start]);
        }
        if !encoded_dir.is_empty() || changed {
          encoded_dir.push_str(&encoded);
          encoded_dir.push('/');
        }
        start = end + 1;
        if changed {
          let mut alias = encoded_dir.clone();
          alias.push_str(&file[start..]);
          #[allow(
            clippy::map_entry,
            reason = "Avoid instantiating the BTreeMap entry API for rare URL aliases"
          )]
          if !integrity.contains_key(&alias) {
            integrity.insert(alias, hash.clone());
          }
        }
      }
      // Keep the original URL if it also appeared as an encoding alias.
      integrity.insert(url, hash.clone());
    }
  }
  Ok(())
}
