use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rspack_core::rspack_sources::{MappingSource, Source};
use rspack_util::base64;
use sha2::{Digest, Sha256, Sha384, Sha512};

/// Source map JSON can feed the digest directly without a full text allocation.
pub fn compute_source_integrity(
  hash_func_names: &Vec<SubresourceIntegrityHashFunction>,
  source: &dyn Source,
) -> String {
  if !source.as_any().is::<MappingSource>() {
    return compute_integrity(hash_func_names, &source.source().into_string_lossy());
  }
  hash_func_names
    .par_iter()
    .map(|hash_func| match hash_func {
      SubresourceIntegrityHashFunction::Sha256 => hash_source::<Sha256>("sha256", source),
      SubresourceIntegrityHashFunction::Sha384 => hash_source::<Sha384>("sha384", source),
      SubresourceIntegrityHashFunction::Sha512 => hash_source::<Sha512>("sha512", source),
    })
    .intersperse(" ".to_string())
    .collect()
}

fn hash_source<D: Digest + Default>(name: &str, source: &dyn Source) -> String {
  use std::io::{self, Write};
  struct DigestWriter<D>(D);
  impl<D: Digest> Write for DigestWriter<D> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
      self.0.update(bytes);
      Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
      Ok(())
    }
  }
  let mut digest = DigestWriter(D::default());
  {
    let mut writer = io::BufWriter::with_capacity(64 * 1024, &mut digest);
    source
      .to_writer(&mut writer)
      .expect("write source to integrity digest");
    writer.flush().expect("flush integrity digest");
  }
  format!("{name}-{}", base64::encode_to_string(digest.0.finalize()))
}

// https://www.w3.org/TR/2016/REC-SRI-20160623/#cryptographic-hash-functions
#[rspack_cacheable::cacheable]
#[derive(Debug, Clone, Copy)]
pub enum SubresourceIntegrityHashFunction {
  Sha256,
  Sha384,
  Sha512,
}

impl TryFrom<String> for SubresourceIntegrityHashFunction {
  type Error = rspack_error::Error;

  fn try_from(value: String) -> Result<Self, rspack_error::Error> {
    match value.as_str() {
      "sha256" => Ok(Self::Sha256),
      "sha384" => Ok(Self::Sha384),
      "sha512" => Ok(Self::Sha512),
      _ => Err(rspack_error::Error::error(format!(
        "Expect SRI hash function to be 'sha256', 'sha384' or 'sha512', but got '{value}'."
      ))),
    }
  }
}

pub fn compute_integrity(
  hash_func_names: &Vec<SubresourceIntegrityHashFunction>,
  source: &str,
) -> String {
  hash_func_names
    .par_iter()
    .map(|hash_func| create_hash(hash_func, source))
    .intersperse(" ".to_string())
    .collect()
}

fn create_hash(hash_func: &SubresourceIntegrityHashFunction, source: &str) -> String {
  match hash_func {
    SubresourceIntegrityHashFunction::Sha256 => {
      let mut hasher = Sha256::new();
      hasher.update(source);
      let digest = &hasher.finalize()[..];
      format!("sha256-{}", base64::encode_to_string(digest))
    }
    SubresourceIntegrityHashFunction::Sha384 => {
      let mut hasher = Sha384::new();
      hasher.update(source);
      let digest = &hasher.finalize()[..];
      format!("sha384-{}", base64::encode_to_string(digest))
    }
    SubresourceIntegrityHashFunction::Sha512 => {
      let mut hasher = Sha512::new();
      hasher.update(source);
      let digest = &hasher.finalize()[..];
      format!("sha512-{}", base64::encode_to_string(digest))
    }
  }
}
