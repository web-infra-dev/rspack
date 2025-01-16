use std::collections::HashMap;

use futures::future::BoxFuture;
use rspack_error::Result;
use rspack_paths::{Utf8Path, Utf8PathBuf};

use crate::is_metadata_route::{PossibleImageFileNameConvention, STATIC_METADATA_IMAGES};

type MetadataResolve =
  dyn Fn(&Utf8Path, &str, &[&str]) -> BoxFuture<'static, Result<Option<Utf8PathBuf>>> + Sync + Send;

pub async fn enum_metadata_files(
  dir: &Utf8Path,
  filename: &str,
  extensions: &[&str],
  metadata_resolve: &MetadataResolve,
  numeric_suffix: bool,
) -> Result<Vec<Utf8PathBuf>> {
  let mut collected_files = Vec::new();

  // Collect <filename>.<ext>, <filename>[].<ext>
  let mut possible_file_names = vec![filename.to_string()];
  if numeric_suffix {
    for index in 0..10 {
      possible_file_names.push(format!("{}{}", filename, index));
    }
  }

  for name in possible_file_names {
    if let Some(resolved) = metadata_resolve(dir, &name, extensions).await? {
      collected_files.push(resolved);
    }
  }

  Ok(collected_files)
}

pub type CollectingMetadata = HashMap<PossibleImageFileNameConvention, Vec<String>>;

struct MetadataImage {
  filename: &'static str,
  extensions: &'static [&'static str],
}

type StaticMetadataImages = HashMap<PossibleImageFileNameConvention, MetadataImage>;

struct WebpackResourceQueries {
  pub edge_ssr_entry: &'static str,
  pub metadata: &'static str,
  pub metadata_route: &'static str,
  pub metadata_image_meta: &'static str,
}

const WEBPACK_RESOURCE_QUERIES: WebpackResourceQueries = WebpackResourceQueries {
  edge_ssr_entry: "__next_edge_ssr_entry__",
  metadata: "__next_metadata__",
  metadata_route: "__next_metadata_route__",
  metadata_image_meta: "__next_metadata_image_meta__",
};

pub struct StaticMetadataCreator<'a> {
  resolved_dir: &'a Utf8Path,
  segment: &'a str,
  is_root_layout_or_root_page: bool,
  page_extensions: &'a [String],
  metadata_resolve: &'a MetadataResolve,
  base_path: &'a Utf8Path,

  // state
  has_static_metadata_files: bool,
  static_images_metadata: CollectingMetadata,
}

impl<'a> StaticMetadataCreator<'a> {
  pub fn new(
    resolved_dir: &'a Utf8Path,
    segment: &'a str,
    metadata_resolve: &'a MetadataResolve,
    is_root_layout_or_root_page: bool,
    page_extensions: &'a [String],
    base_path: &'a Utf8Path,
  ) -> Self {
    Self {
      resolved_dir,
      segment,
      is_root_layout_or_root_page,
      page_extensions,
      base_path,
      metadata_resolve,

      has_static_metadata_files: false,
      static_images_metadata: Default::default(),
    }
  }

  async fn collect_icon_module_if_exists(
    &mut self,
    ty: PossibleImageFileNameConvention,
  ) -> Result<()> {
    if matches!(ty, PossibleImageFileNameConvention::Manifest) {
      let mut static_manifest_extension = Vec::with_capacity(self.page_extensions.len() + 2);
      static_manifest_extension.push("webmanifest");
      static_manifest_extension.push("json");
      for page_extension in self.page_extensions {
        static_manifest_extension.push(page_extension);
      }

      let manifest_file = enum_metadata_files(
        &self.resolved_dir,
        "manifest",
        &static_manifest_extension,
        self.metadata_resolve,
        false,
      )
      .await?;
      if manifest_file.len() > 0 {
        self.has_static_metadata_files = true;
        let path_buf = &manifest_file[0];
        let Some(name) = path_buf.file_stem() else {
          return Ok(());
        };
        let Some(ext) = path_buf.extension() else {
          return Ok(());
        };
        let extension = if static_manifest_extension
          .iter()
          .any(|the_ext| ext == *the_ext)
        {
          ext
        } else {
          "webmanifest"
        };
        self.static_images_metadata.insert(
          PossibleImageFileNameConvention::Manifest,
          vec![json::stringify(format!("/{}.{}", name, extension))],
        );
      }
      return Ok(());
    }

    let is_favicon = matches!(ty, PossibleImageFileNameConvention::Favicon);
    let metadata = STATIC_METADATA_IMAGES.get(&ty).unwrap();
    let mut extensions = metadata.extensions.to_vec();
    if !is_favicon {
      self
        .page_extensions
        .iter()
        .for_each(|ext| extensions.push(ext));
    }
    let mut resolved_metadata_files = enum_metadata_files(
      &self.resolved_dir,
      &metadata.filename,
      &extensions,
      self.metadata_resolve,
      !is_favicon,
    )
    .await?;

    resolved_metadata_files.sort_by(|a, b| a.cmp(b));

    for filepath in resolved_metadata_files {
      let query = format!(
        "{{{}:{},{}:{},{}:{},{}:{}}}",
        json::stringify("type"),
        json::stringify(ty.as_str()),
        json::stringify("segment"),
        json::stringify(self.segment),
        json::stringify("basePath"),
        json::stringify(self.base_path.as_str()),
        json::stringify("pageExtensions"),
        json::stringify(
          self
            .page_extensions
            .iter()
            .map(|ext| json::stringify(ext.as_str()))
            .collect::<Vec<_>>()
            .join(",")
        ),
      );
      let image_module_import_source = format!(
        "next-metadata-image-loader?{}!{}?{}",
        query, filepath, WEBPACK_RESOURCE_QUERIES.metadata
      );

      let image_module = format!(
        "(async (props) => (await import(/* webpackMode: \"eager\" */ {})).default(props))",
        serde_json::to_string(&image_module_import_source).unwrap()
      );

      self.has_static_metadata_files = true;
      if matches!(ty, PossibleImageFileNameConvention::Favicon) {
        let metadata = self
          .static_images_metadata
          .entry(PossibleImageFileNameConvention::Icon)
          .or_insert(vec![]);
        metadata.insert(0, image_module);
      } else {
        let metadata = self.static_images_metadata.entry(ty).or_insert(vec![]);
        metadata.push(image_module);
      }
    }
    todo!()
  }

  pub async fn create_static_metadata_from_route(mut self) -> Result<Option<CollectingMetadata>> {
    // Intentionally make these serial to reuse directory access cache.
    self
      .collect_icon_module_if_exists(PossibleImageFileNameConvention::Icon)
      .await?;
    self
      .collect_icon_module_if_exists(PossibleImageFileNameConvention::Apple)
      .await?;
    self
      .collect_icon_module_if_exists(PossibleImageFileNameConvention::OpenGraph)
      .await?;
    self
      .collect_icon_module_if_exists(PossibleImageFileNameConvention::Twitter)
      .await?;
    if self.is_root_layout_or_root_page {
      self
        .collect_icon_module_if_exists(PossibleImageFileNameConvention::Favicon)
        .await?;
      self
        .collect_icon_module_if_exists(PossibleImageFileNameConvention::Manifest)
        .await?;
    }
    Ok(if self.has_static_metadata_files {
      Some(self.static_images_metadata)
    } else {
      None
    })
  }
}

pub async fn create_static_metadata_from_route<'a>(
  resolved_dir: &'a Utf8Path,
  segment: &'a str,
  metadata_resolve: &'a MetadataResolve,
  is_root_layout_or_root_page: bool,
  page_extensions: &[String],
  base_path: &'a Utf8Path,
) -> Result<Option<CollectingMetadata>> {
  let creator = StaticMetadataCreator::new(
    resolved_dir,
    segment,
    metadata_resolve,
    is_root_layout_or_root_page,
    page_extensions,
    base_path,
  );
  creator.create_static_metadata_from_route().await
}
