use std::{
  hash::{Hash, Hasher},
  io::ErrorKind,
};

use lz4_flex::block::{
  compress_into, compress_into_with_dict, decompress_into, decompress_into_with_dict,
  get_maximum_output_size,
};
use rustc_hash::FxHasher;

use super::{IndexGenerator, Pack, PackId, PackIndex, ScopeFileSystem};
use crate::{Error, Result};

const CHUNK: usize = 256 * 1024;
const DICT: usize = 64 * 1024;

// u64 LE decoded size, then u32 LE length-prefixed LZ4 blocks of 256 KiB decoded each,
// linked by the previous 64 KiB.
// The mode's cache directory selects this format, so no encoding marker is needed.
pub(super) async fn save(pack: &mut Pack, fs: &ScopeFileSystem, id: PackId) -> Result<PackIndex> {
  // Nearby keys share the LZ4 window and make output deterministic.
  // Keys are unique within a pack, so an unstable sort is sufficient.
  pack.data.sort_unstable_by(|a, b| a.0.cmp(&b.0));

  let pack_name = id.pack_name();
  let mut decoded_size = 0usize;
  let mut index_gen = IndexGenerator::default();
  for (key, value) in &pack.data {
    let header = item_header(key, value);
    decoded_size = decoded_size
      .checked_add(header.len())
      .and_then(|len| len.checked_add(key.len()))
      .and_then(|len| len.checked_add(value.len()))
      .ok_or_else(|| invalid(&pack_name, "serialized body size overflow"))?;
    index_gen.add_key(key);
    index_gen.add_value(value);
  }

  // Open before allocating encode buffers: stream_write yields (remove_file),
  // and buffers built before it would stay resident across that await in every
  // concurrently spawned pack save. Default native filesystem writes do not yield;
  // custom intermediate filesystems with awaiting writes retain buffers per in-flight save.
  let mut writer = fs.stream_write(&pack_name).await?;
  write_bytes(writer.as_mut(), &(decoded_size as u64).to_le_bytes()).await?;
  if decoded_size != 0 {
    let mut encoder = Encoder::new(&pack_name, decoded_size)?;
    let mut consumed = 0;
    for (key, value) in &pack.data {
      let header = item_header(key, value);
      for bytes in [header.as_bytes(), key.as_slice(), value.as_slice()] {
        encoder.write(writer.as_mut(), bytes).await?;
        consumed += bytes.len();
      }
    }
    debug_assert_eq!(consumed, decoded_size);
    if encoder.staging.len() > encoder.dict_len {
      encoder.write_chunk(writer.as_mut()).await?;
    }
  }
  writer.flush().await?;
  writer.close().await?;
  Ok(index_gen.finish())
}

fn item_header(key: &[u8], value: &[u8]) -> String {
  format!("{} {}\n", key.len(), value.len())
}

// Use append writes: not every filesystem's write_all appends to the stream.
async fn write_bytes(writer: &mut dyn rspack_fs::WriteStream, mut bytes: &[u8]) -> Result<()> {
  while !bytes.is_empty() {
    let written = writer.write(bytes).await?;
    if written == 0 {
      return Err(Error::FS(rspack_fs::Error::from(std::io::Error::from(
        ErrorKind::WriteZero,
      ))));
    }
    bytes = &bytes[written..];
  }
  Ok(())
}

struct Encoder<'a> {
  pack_name: &'a str,
  staging: Vec<u8>,
  scratch: Vec<u8>,
  dict_len: usize,
}

impl<'a> Encoder<'a> {
  fn new(pack_name: &'a str, decoded_size: usize) -> Result<Self> {
    let allocation_error = |reason| {
      Error::FS(rspack_fs::Error::from(std::io::Error::new(
        ErrorKind::OutOfMemory,
        format!("Cannot allocate pack '{pack_name}': {reason}"),
      )))
    };
    let first = CHUNK.min(decoded_size);
    // Later chunks need a dictionary plus their input; small packs need only
    // their body. This capacity also fits a short final chunk with its dictionary.
    let staging_len = first + DICT.min(decoded_size - first);
    let mut staging = Vec::new();
    staging
      .try_reserve_exact(staging_len)
      .map_err(allocation_error)?;
    let scratch_len = get_maximum_output_size(first);
    let mut scratch = Vec::new();
    scratch
      .try_reserve_exact(scratch_len)
      .map_err(allocation_error)?;
    scratch.resize(scratch_len, 0);
    Ok(Self {
      pack_name,
      staging,
      scratch,
      dict_len: 0,
    })
  }

  async fn write(
    &mut self,
    writer: &mut dyn rspack_fs::WriteStream,
    mut bytes: &[u8],
  ) -> Result<()> {
    while !bytes.is_empty() {
      let n = bytes.len().min(self.dict_len + CHUNK - self.staging.len());
      self.staging.extend_from_slice(&bytes[..n]);
      bytes = &bytes[n..];
      if self.staging.len() == self.dict_len + CHUNK {
        self.write_chunk(writer).await?;
      }
    }
    Ok(())
  }

  async fn write_chunk(&mut self, writer: &mut dyn rspack_fs::WriteStream) -> Result<()> {
    let (dict, chunk) = self.staging.split_at(self.dict_len);
    // Scratch uses get_maximum_output_size, so encoding cannot fail for lack of space.
    // Output is at most 288,378 bytes, so casting its length to u32 cannot truncate.
    let written = if dict.is_empty() {
      compress_into(chunk, &mut self.scratch)
    } else {
      compress_into_with_dict(chunk, &mut self.scratch, dict)
    }
    .map_err(|_| invalid(self.pack_name, "LZ4 encode failed"))?;
    write_bytes(writer, &(written as u32).to_le_bytes()).await?;
    write_bytes(writer, &self.scratch[..written]).await?;
    let staged_len = self.staging.len();
    self.dict_len = DICT.min(staged_len);
    self.staging.copy_within(staged_len - self.dict_len.., 0);
    self.staging.truncate(self.dict_len);
    Ok(())
  }
}

// The "{key_len} {value_len}" header needs at most 41 bytes on 64-bit platforms.
// Leave room for leading zeroes without allowing corrupt packs to grow an unbounded header.
const MAX_HEADER: usize = 128;

#[cold]
fn invalid(pack_name: &str, reason: &str) -> Error {
  Error::InvalidFormat(format!("Invalid pack '{pack_name}': {reason}"))
}

enum State {
  Header,
  Key,
  Value,
}

struct ItemParser<'a> {
  pack_name: &'a str,
  remaining: usize,
  state: State,
  header: Vec<u8>,
  key_len: usize,
  value_len: usize,
  key: Vec<u8>,
  value: Vec<u8>,
  data: Vec<(Vec<u8>, Vec<u8>)>,
  hasher: FxHasher,
}

impl<'a> ItemParser<'a> {
  fn new(pack_name: &'a str, remaining: usize) -> Self {
    Self {
      pack_name,
      remaining,
      state: State::Header,
      header: Vec::new(),
      key_len: 0,
      value_len: 0,
      key: Vec::new(),
      value: Vec::new(),
      data: Vec::new(),
      hasher: FxHasher::default(),
    }
  }

  fn start_item(&mut self) -> Result<()> {
    let header = std::str::from_utf8(&self.header)
      .map_err(|_| invalid(self.pack_name, "non-UTF8 item header"))?;
    let (key_len, value_len) = header
      .split_once(' ')
      .ok_or_else(|| invalid(self.pack_name, "expected key_len value_len"))?;
    self.key_len = key_len
      .parse::<usize>()
      .map_err(|_| invalid(self.pack_name, "invalid key length"))?;
    self.value_len = value_len
      .parse::<usize>()
      .map_err(|_| invalid(self.pack_name, "invalid value length"))?;
    self
      .key_len
      .checked_add(self.value_len)
      .filter(|&len| len <= self.remaining)
      .ok_or_else(|| invalid(self.pack_name, "item lengths exceed remaining body"))?;
    self
      .key
      .try_reserve_exact(self.key_len)
      .map_err(|_| invalid(self.pack_name, "cannot allocate item key"))?;
    self
      .value
      .try_reserve_exact(self.value_len)
      .map_err(|_| invalid(self.pack_name, "cannot allocate item value"))?;
    self.header.clear();
    self.state = State::Key;
    Ok(())
  }

  fn feed(&mut self, mut bytes: &[u8]) -> Result<()> {
    loop {
      match self.state {
        State::Header => {
          if bytes.is_empty() {
            break;
          }
          let newline = bytes.iter().position(|&byte| byte == b'\n');
          let n = newline.unwrap_or(bytes.len());
          if n > MAX_HEADER - self.header.len() {
            return Err(invalid(self.pack_name, "item header too long"));
          }
          self.header.extend_from_slice(&bytes[..n]);
          let consumed = n + usize::from(newline.is_some());
          bytes = &bytes[consumed..];
          self.remaining -= consumed;
          if newline.is_some() {
            self.start_item()?;
          }
        }
        State::Key => {
          let n = bytes.len().min(self.key_len - self.key.len());
          self.key.extend_from_slice(&bytes[..n]);
          bytes = &bytes[n..];
          self.remaining -= n;
          if self.key.len() == self.key_len {
            self.state = State::Value;
          } else {
            break;
          }
        }
        State::Value => {
          let n = bytes.len().min(self.value_len - self.value.len());
          self.value.extend_from_slice(&bytes[..n]);
          bytes = &bytes[n..];
          self.remaining -= n;
          if self.value.len() == self.value_len {
            // Hash the final Vecs, including their lengths, just as save does.
            self.key.hash(&mut self.hasher);
            self.value.hash(&mut self.hasher);
            self.data.push((
              std::mem::take(&mut self.key),
              std::mem::take(&mut self.value),
            ));
            self.state = State::Header;
          } else {
            break;
          }
        }
      }
    }
    Ok(())
  }

  fn finish(self) -> Result<(Pack, u64)> {
    if !self.header.is_empty() {
      return Err(invalid(self.pack_name, "unterminated item header"));
    }
    if self.remaining != 0 || !matches!(self.state, State::Header) {
      return Err(invalid(
        self.pack_name,
        "item lengths exceed remaining body",
      ));
    }
    Ok((Pack { data: self.data }, self.hasher.finish()))
  }
}

pub(super) async fn load(fs: &ScopeFileSystem, id: PackId) -> Result<(Pack, u64)> {
  let pack_name = id.pack_name();
  let file_size = fs.stat(&pack_name).await?.size;
  if file_size < 8 {
    return Err(invalid(&pack_name, "encoded size out of bounds"));
  }
  let file_size =
    usize::try_from(file_size).map_err(|_| invalid(&pack_name, "encoded size out of bounds"))?;
  let mut reader = fs.stream_read(&pack_name).await?;
  let size_bytes = reader
    .read(8)
    .await?
    .try_into()
    .map_err(|_| invalid(&pack_name, "invalid LZ4 decoded size"))?;
  let decoded_size = usize::try_from(u64::from_le_bytes(size_bytes))
    .map_err(|_| invalid(&pack_name, "LZ4 decoded size out of bounds"))?;
  let mut file_remaining = file_size - 8;
  // Length extensions consume a byte per 255 decoded bytes.
  if decoded_size > file_remaining.saturating_mul(255) {
    return Err(invalid(&pack_name, "LZ4 decoded size out of bounds"));
  }

  // Allocate after stat, the only yielding native call; awaiting custom reads retain this window.
  let first = CHUNK.min(decoded_size);
  let window_len = first + DICT.min(decoded_size - first);
  let mut window = Vec::new();
  window
    .try_reserve_exact(window_len)
    .map_err(|_| invalid(&pack_name, "cannot allocate decode window"))?;
  window.resize(window_len, 0);
  let mut parser = ItemParser::new(&pack_name, decoded_size);
  let mut decoded_remaining = decoded_size;
  let mut dict_len = 0;
  while decoded_remaining > 0 {
    if file_remaining < 4 {
      return Err(invalid(&pack_name, "missing LZ4 chunk length"));
    }
    let len_bytes = reader
      .read(4)
      .await?
      .try_into()
      .map_err(|_| invalid(&pack_name, "invalid LZ4 chunk length"))?;
    file_remaining -= 4;
    let len = u32::from_le_bytes(len_bytes) as usize;
    let n = CHUNK.min(decoded_remaining);
    if len == 0 || len > file_remaining || len > get_maximum_output_size(n) {
      return Err(invalid(&pack_name, "LZ4 chunk length out of bounds"));
    }
    let bytes = reader.read(len).await?;
    if bytes.len() != len {
      return Err(invalid(&pack_name, "short LZ4 read"));
    }
    file_remaining -= len;
    let decoded_end = dict_len + n;
    let (dict, out) = window[..decoded_end].split_at_mut(dict_len);
    let written = if dict.is_empty() {
      decompress_into(&bytes, out)
    } else {
      decompress_into_with_dict(&bytes, out, dict)
    }
    .map_err(|_| invalid(&pack_name, "LZ4 decode failed"))?;
    if written != n {
      return Err(invalid(&pack_name, "LZ4 decoded chunk size mismatch"));
    }
    parser.feed(out)?;
    dict_len = DICT.min(decoded_end);
    window.copy_within(decoded_end - dict_len..decoded_end, 0);
    decoded_remaining -= n;
  }
  if file_remaining != 0 {
    return Err(invalid(&pack_name, "trailing bytes after LZ4 chunks"));
  }
  // Free the decode window before awaiting close; only the parsed items outlive the stream.
  drop(window);
  reader.close().await?;
  parser.finish()
}
