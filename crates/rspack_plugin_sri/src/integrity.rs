use std::io::{self, Write};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rspack_core::rspack_sources::{MappingSource, Source};
use rspack_plugin_real_content_hash::RealContentHashSource;
use rspack_util::base64;
use sha2::{Digest, Sha256, Sha384, Sha512};

/// Source map JSON can feed the digest directly without a full text allocation.
pub fn compute_source_integrity(
  hash_func_names: &Vec<SubresourceIntegrityHashFunction>,
  source: &dyn Source,
) -> String {
  if !source.as_any().is::<MappingSource>()
    && !source
      .as_any()
      .downcast_ref::<RealContentHashSource>()
      .is_some_and(RealContentHashSource::can_stream_as_text)
  {
    return compute_integrity(hash_func_names, &source.source().into_string_lossy());
  }
  if hash_func_names.is_empty() {
    return String::new();
  }
  let mut digests = IntegrityWriter::default();
  for hash_func in hash_func_names {
    match hash_func {
      SubresourceIntegrityHashFunction::Sha256 => {
        digests.sha256.get_or_insert_with(Sha256::new);
      }
      SubresourceIntegrityHashFunction::Sha384 => {
        digests.sha384.get_or_insert_with(Sha384::new);
      }
      SubresourceIntegrityHashFunction::Sha512 => {
        digests.sha512.get_or_insert_with(Sha512::new);
      }
    }
  }
  {
    let mut writer = io::BufWriter::with_capacity(64 * 1024, &mut digests);
    source
      .to_writer(&mut writer)
      .expect("write source to integrity digest");
    writer.flush().expect("flush integrity digest");
  }
  let sha256 = digests.sha256.map(finish_sha256);
  let sha384 = digests.sha384.map(finish_sha384);
  let sha512 = digests.sha512.map(finish_sha512);
  let mut integrity = String::new();
  for hash_func in hash_func_names {
    let digest = match hash_func {
      SubresourceIntegrityHashFunction::Sha256 => &sha256,
      SubresourceIntegrityHashFunction::Sha384 => &sha384,
      SubresourceIntegrityHashFunction::Sha512 => &sha512,
    }
    .as_deref()
    .expect("configured integrity digest must be initialized");
    if !integrity.is_empty() {
      integrity.push(' ');
    }
    integrity.push_str(digest);
  }
  integrity
}

// Serialize each source map once, including when several algorithms are used.
// Parallelism is provided by the callers that process assets in parallel.
#[derive(Default)]
struct IntegrityWriter {
  sha256: Option<Sha256>,
  sha384: Option<Sha384>,
  sha512: Option<Sha512>,
}

impl Write for IntegrityWriter {
  fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
    if let Some(digest) = &mut self.sha256 {
      digest.update(bytes);
    }
    if let Some(digest) = &mut self.sha384 {
      digest.update(bytes);
    }
    if let Some(digest) = &mut self.sha512 {
      digest.update(bytes);
    }
    Ok(bytes.len())
  }

  fn flush(&mut self) -> io::Result<()> {
    Ok(())
  }
}

fn finish_sha256(hasher: Sha256) -> String {
  let digest = hasher.finalize();
  format!("sha256-{}", base64::encode_to_string(&digest[..]))
}

fn finish_sha384(hasher: Sha384) -> String {
  let digest = hasher.finalize();
  format!("sha384-{}", base64::encode_to_string(&digest[..]))
}

fn finish_sha512(hasher: Sha512) -> String {
  let digest = hasher.finalize();
  format!("sha512-{}", base64::encode_to_string(&digest[..]))
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
      finish_sha256(hasher)
    }
    SubresourceIntegrityHashFunction::Sha384 => {
      let mut hasher = Sha384::new();
      hasher.update(source);
      finish_sha384(hasher)
    }
    SubresourceIntegrityHashFunction::Sha512 => {
      let mut hasher = Sha512::new();
      hasher.update(source);
      finish_sha512(hasher)
    }
  }
}
