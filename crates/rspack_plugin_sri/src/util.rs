use std::{borrow::Cow, sync::LazyLock};

use cow_utils::CowUtils;
use indexmap::IndexSet;
use rspack_core::{AssetInfo, ChunkGroupUkey, ChunkUkey, Compilation, ManifestAssetType};

use crate::{SubresourceIntegrityHashFunction, integrity::compute_integrity};

pub const SRI_HASH_VARIABLE_REFERENCE: &str = "__webpack_require__.sriHashes";
pub const SRI_HASH_CSS_VARIABLE_REFERENCE: &str = "__webpack_require__.sriCssHashes";
pub const SRI_HASH_EXTRACT_CSS_VARIABLE_REFERENCE: &str = "__webpack_require__.sriExtractCssHashes";

pub const PLACEHOLDER_PREFIX: &str = "*-*-*-CHUNK-SRI-HASH-";

pub static PLACEHOLDER_REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
  let escaped_prefix = regex::escape(PLACEHOLDER_PREFIX);
  regex::Regex::new(&format!(
    r"{escaped_prefix}[a-zA-Z0-9=/+]+(\s+sha\d{{3}}-[a-zA-Z0-9=/+]+)*"
  ))
  .expect("should initialize `Regex`")
});

pub fn find_chunks(chunk: &ChunkUkey, compilation: &Compilation) -> IndexSet<ChunkUkey> {
  let mut all_chunks = IndexSet::default();
  let mut visited_groups = IndexSet::default();
  recurse_chunk(chunk, &mut all_chunks, &mut visited_groups, compilation);
  all_chunks
}

fn recurse_chunk_group(
  group: &ChunkGroupUkey,
  all_chunks: &mut IndexSet<ChunkUkey>,
  visited_groups: &mut IndexSet<ChunkGroupUkey>,
  compilation: &Compilation,
) {
  if visited_groups.contains(group) {
    return;
  }
  visited_groups.insert(*group);

  if let Some(chunk_group) = compilation.chunk_group_by_ukey.get(group) {
    for chunk in chunk_group.chunks.iter() {
      recurse_chunk(chunk, all_chunks, visited_groups, compilation);
    }
    for child in chunk_group.children.iter() {
      recurse_chunk_group(child, all_chunks, visited_groups, compilation);
    }
  }
}

fn recurse_chunk(
  chunk: &ChunkUkey,
  all_chunks: &mut IndexSet<ChunkUkey>,
  visited_groups: &mut IndexSet<ChunkGroupUkey>,
  compilation: &Compilation,
) {
  if all_chunks.contains(chunk) {
    return;
  }
  all_chunks.insert(*chunk);

  if let Some(chunk) = compilation.chunk_by_ukey.get(chunk) {
    for group in chunk.groups() {
      recurse_chunk_group(group, all_chunks, visited_groups, compilation);
    }
  }
}

pub fn make_placeholder(
  asset_type: ManifestAssetType,
  hash_funcs: &Vec<SubresourceIntegrityHashFunction>,
  id: &str,
) -> String {
  let placeholder_source = format!("{PLACEHOLDER_PREFIX}{asset_type}{id}");
  let filler = compute_integrity(hash_funcs, &placeholder_source);
  format!(
    "{}{}",
    PLACEHOLDER_PREFIX,
    &filler[PLACEHOLDER_PREFIX.len()..]
  )
}

pub fn normalize_path(path: &str) -> Cow<'_, str> {
  path.split('?').next().unwrap_or("").cow_replace('\\', "/")
}

pub fn use_any_hash(info: &AssetInfo) -> bool {
  !info.chunk_hash.is_empty() || !info.full_hash.is_empty() || !info.content_hash.is_empty()
}
