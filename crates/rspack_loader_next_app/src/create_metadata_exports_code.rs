use crate::{
  create_static_metadata_from_route::CollectingMetadata,
  is_metadata_route::PossibleImageFileNameConvention,
};

pub fn create_metadata_exports_code(metadata: &Option<CollectingMetadata>) -> String {
  if let Some(metadata) = metadata {
    format!(
      "{}: {{
    icon: [{}],
    apple: [{}],
    openGraph: [{}],
    twitter: [{}],
    manifest: {}
  }}",
      METADATA_TYPE,
      &metadata
        .get(&PossibleImageFileNameConvention::Icon)
        .unwrap_or(&vec![])
        .join(","),
      &metadata
        .get(&PossibleImageFileNameConvention::Apple)
        .unwrap_or(&vec![])
        .join(","),
      &metadata
        .get(&PossibleImageFileNameConvention::OpenGraph)
        .unwrap_or(&vec![])
        .join(","),
      &metadata
        .get(&PossibleImageFileNameConvention::Twitter)
        .unwrap_or(&vec![])
        .join(","),
      metadata
        .get(&PossibleImageFileNameConvention::Manifest)
        .map(|vec| vec.get(0))
        .flatten()
        .map(String::as_str)
        .unwrap_or_else(|| "undefined")
    )
  } else {
    String::new()
  }
}

const METADATA_TYPE: &str = "metadata";
