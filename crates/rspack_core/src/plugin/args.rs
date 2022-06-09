use crate::{Compilation, ResolveKind};

#[derive(Debug)]
pub struct ParseModuleArgs<'a> {
  pub uri: &'a str,
  pub source: String,
}

#[derive(Debug, Clone)]
pub struct RenderManifestArgs<'me> {
  pub chunk_id: &'me str,
  pub compilation: &'me Compilation,
}

pub struct ResolveArgs<'a> {
  pub importer: Option<&'a str>,
  pub specifier: &'a str,
  pub kind: ResolveKind,
}

pub struct LoadArgs<'a> {
  pub uri: &'a str,
}

pub struct TransformArgs {
  pub source: String,
}
