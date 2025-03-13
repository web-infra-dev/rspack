use std::collections::HashMap;

use rspack_core::{CompilationId, LoaderContext, RunnerContext};
use rspack_error::Result;
use rspack_paths::Utf8PathBuf;
use rspack_util::fx_hash::BuildFxHasher;
use serde_json::json;

use crate::{
  is_metadata_route::{PossibleImageFileNameConvention, STATIC_METADATA_IMAGES},
  util::metadata_resolver,
};

pub async fn enum_metadata_files(
  dir: &str,
  filename: &str,
  extensions: &[&str],
  numeric_suffix: bool,
  app_dir: &str,
  loader_context: &mut LoaderContext<RunnerContext>,
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
    let (resolved, missing_dependencies) = metadata_resolver(
      dir,
      &name,
      extensions,
      app_dir,
      loader_context.context.compilation_id,
    )
    .await?;
    loader_context
      .missing_dependencies
      .extend(missing_dependencies);
    if let Some(resolved) = resolved {
      collected_files.push(Utf8PathBuf::from(resolved));
    }
  }

  Ok(collected_files)
}

pub type CollectingMetadata = HashMap<PossibleImageFileNameConvention, Vec<String>, BuildFxHasher>;

struct MetadataImage {
  filename: &'static str,
  extensions: &'static [&'static str],
}

type StaticMetadataImages = HashMap<PossibleImageFileNameConvention, MetadataImage, BuildFxHasher>;

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
  resolved_dir: &'a str,
  segment: &'a str,
  is_root_layout_or_root_page: bool,
  page_extensions: &'a [String],
  base_path: &'a str,
  app_dir: &'a str,
  loader_context: &'a mut LoaderContext<RunnerContext>,

  // state
  has_static_metadata_files: bool,
  static_images_metadata: CollectingMetadata,
}

impl<'a> StaticMetadataCreator<'a> {
  pub fn new(
    resolved_dir: &'a str,
    segment: &'a str,
    is_root_layout_or_root_page: bool,
    page_extensions: &'a [String],
    base_path: &'a str,
    app_dir: &'a str,
    loader_context: &'a mut LoaderContext<RunnerContext>,
  ) -> Self {
    Self {
      resolved_dir,
      segment,
      is_root_layout_or_root_page,
      page_extensions,
      base_path,
      app_dir,
      loader_context,

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
        false,
        &self.app_dir,
        self.loader_context,
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
      !is_favicon,
      &self.app_dir,
      self.loader_context,
    )
    .await?;

    resolved_metadata_files.sort_by(|a, b| a.cmp(b));

    for filepath in resolved_metadata_files {
      let query = json!({
        "type": ty.as_str(),
        "segment": self.segment,
        "basePath": self.base_path,
        "pageExtensions": self
        .page_extensions
      })
      .to_string();
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
    Ok(())
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

pub async fn create_static_metadata_from_route(
  resolved_dir: &str,
  segment: &str,
  is_root_layout_or_root_page: bool,
  page_extensions: &[String],
  base_path: &str,
  app_dir: &str,
  loader_context: &mut LoaderContext<RunnerContext>,
) -> Result<Option<CollectingMetadata>> {
  let creator = StaticMetadataCreator::new(
    resolved_dir,
    segment,
    is_root_layout_or_root_page,
    page_extensions,
    base_path,
    app_dir,
    loader_context,
  );
  creator.create_static_metadata_from_route().await
}
