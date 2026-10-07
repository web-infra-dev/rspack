use lz4_flex::block::{compress_into, decompress_into, get_maximum_output_size};
use rspack_cacheable::rkyv::util::AlignedVec;
use rspack_error::{Result, error};

const CHUNK_SIZE: usize = 256 * 1024;
const RAW_CHUNK: u32 = 0x8000_0000;
const RETAINED_SLACK: usize = 64;

/// Cache value format: `u32 LE decoded_len`, then independent 256 KiB chunks.
/// Each chunk starts with a `u32 LE word`: the high bit marks raw bytes and the
/// remaining bits give their length; otherwise the word is the LZ4 block length.
/// Raw blocks follow the LZ4 frame format's uncompressed-block convention.
/// Each chunk decodes to `min(256 KiB, remaining)` bytes, with no dictionary.
/// Empty values have only the size header; truncation and trailing bytes fail.
pub(super) fn encode(bytes: AlignedVec) -> Result<Vec<u8>> {
  let decoded_len = u32::try_from(bytes.len()).map_err(rspack_error::Error::from_error)?;
  let capacity = bytes
    .len()
    .checked_add(4 * bytes.len().div_ceil(CHUNK_SIZE))
    .and_then(|size| size.checked_add(4))
    .ok_or_else(|| error!("compressed cache value is too large"))?;
  // Raw chunks cap the output at the archive size plus its chunk headers.
  let mut output = Vec::new();
  output
    .try_reserve_exact(capacity)
    .map_err(rspack_error::Error::from_error)?;
  output.extend_from_slice(&decoded_len.to_le_bytes());
  let mut scratch = vec![0; get_maximum_output_size(bytes.len().min(CHUNK_SIZE))];
  for chunk in bytes.chunks(CHUNK_SIZE) {
    let compressed_len =
      compress_into(chunk, &mut scratch).map_err(rspack_error::Error::from_error)?;
    if compressed_len >= chunk.len() {
      output.extend_from_slice(&(RAW_CHUNK | chunk.len() as u32).to_le_bytes());
      output.extend_from_slice(chunk);
    } else {
      output.extend_from_slice(&(compressed_len as u32).to_le_bytes());
      output.extend_from_slice(&scratch[..compressed_len]);
    }
  }
  drop(scratch);
  drop(bytes);
  // mimalloc may retain a block on shrink, so copy only after releasing the archive.
  if output.capacity() - output.len() > RETAINED_SLACK {
    Ok(output.as_slice().to_vec())
  } else {
    Ok(output)
  }
}

pub(super) fn decode(mut input: &[u8]) -> Result<AlignedVec> {
  let input_len = input.len();
  let decoded_len = read_word(&mut input)? as usize;
  // LZ4's maximum expansion is 255:1. Reject corrupt sizes before allocating.
  if decoded_len > input_len.saturating_mul(255) {
    return Err(error!("compressed cache value decoded size is too large"));
  }
  // Decode into an aligned buffer so deserialization reads it without a copy.
  let mut output = AlignedVec::with_capacity(decoded_len);
  output.resize(decoded_len, 0);
  for chunk in output.as_mut_slice().chunks_mut(CHUNK_SIZE) {
    let word = read_word(&mut input)?;
    let block_len = (word & !RAW_CHUNK) as usize;
    let raw = word & RAW_CHUNK != 0;
    if (raw && block_len != chunk.len())
      || (!raw && (block_len == 0 || block_len > get_maximum_output_size(chunk.len())))
    {
      return Err(error!("compressed cache value has an invalid chunk length"));
    }
    let block = take(&mut input, block_len)?;
    if raw {
      chunk.copy_from_slice(block);
    } else {
      let written = decompress_into(block, chunk).map_err(rspack_error::Error::from_error)?;
      if written != chunk.len() {
        return Err(error!("compressed cache value chunk decoded size mismatch"));
      }
    }
  }
  if !input.is_empty() {
    return Err(error!("compressed cache value has trailing bytes"));
  }
  Ok(output)
}

fn read_word(input: &mut &[u8]) -> Result<u32> {
  let bytes = take(input, 4)?;
  Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn take<'a>(input: &mut &'a [u8], len: usize) -> Result<&'a [u8]> {
  let (bytes, rest) = input
    .split_at_checked(len)
    .ok_or_else(|| error!("compressed cache value is truncated"))?;
  *input = rest;
  Ok(bytes)
}
