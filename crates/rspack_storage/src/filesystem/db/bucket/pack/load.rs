use std::{
  fmt::Arguments,
  hash::{Hash, Hasher},
};

use lz4_flex::block::{decompress_into_with_dict, get_maximum_output_size};
use rustc_hash::FxHasher;

use super::{CHUNK, DICT, LZ4_ENCODING, Pack, PackId, RAW_ENCODING, ScopeFileSystem};
use crate::{Error, Result};

// The "{key_len} {value_len}" header needs at most 41 bytes on 64-bit platforms.
// Leave room for leading zeroes without allowing corrupt packs to grow an unbounded header.
const MAX_HEADER: usize = 128;

#[cold]
#[inline(never)]
fn invalid(pack_name: &str, reason: Arguments<'_>) -> Error {
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
      .map_err(|e| invalid(self.pack_name, format_args!("non-UTF8 item header: {e}")))?;
    let (key_len, value_len) = header.split_once(' ').ok_or_else(|| {
      invalid(
        self.pack_name,
        format_args!("expected key_len value_len, got '{header:.128}'"),
      )
    })?;
    self.key_len = key_len.parse::<usize>().map_err(|e| {
      invalid(
        self.pack_name,
        format_args!("invalid key length '{key_len:.128}': {e}"),
      )
    })?;
    self.value_len = value_len.parse::<usize>().map_err(|e| {
      invalid(
        self.pack_name,
        format_args!("invalid value length '{value_len:.128}': {e}"),
      )
    })?;
    self
      .key_len
      .checked_add(self.value_len)
      .filter(|&len| len <= self.remaining)
      .ok_or_else(|| {
        invalid(
          self.pack_name,
          format_args!("item lengths exceed remaining body"),
        )
      })?;
    self.key.try_reserve_exact(self.key_len).map_err(|e| {
      invalid(
        self.pack_name,
        format_args!("cannot allocate item key: {e}"),
      )
    })?;
    self.value.try_reserve_exact(self.value_len).map_err(|e| {
      invalid(
        self.pack_name,
        format_args!("cannot allocate item value: {e}"),
      )
    })?;
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
            return Err(invalid(
              self.pack_name,
              format_args!("item header too long"),
            ));
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
      return Err(invalid(
        self.pack_name,
        format_args!("unterminated item header"),
      ));
    }
    if self.remaining != 0 || !matches!(self.state, State::Header) {
      return Err(invalid(
        self.pack_name,
        format_args!("item lengths exceed remaining body"),
      ));
    }
    Ok((Pack { data: self.data }, self.hasher.finish()))
  }
}

pub(super) async fn load(fs: &ScopeFileSystem, id: PackId) -> Result<(Pack, u64)> {
  let pack_name = id.pack_name();
  let file_size = fs.stat(&pack_name).await?.size;
  if file_size == 0 {
    return Err(invalid(
      &pack_name,
      format_args!("encoded size out of bounds"),
    ));
  }
  let file_size = usize::try_from(file_size)
    .map_err(|e| invalid(&pack_name, format_args!("encoded size out of bounds: {e}")))?;
  let mut reader = fs.stream_read(&pack_name).await?;
  let encoding = reader.read(1).await?;
  let encoding = *encoding
    .first()
    .ok_or_else(|| invalid(&pack_name, format_args!("missing encoding")))?;
  let mut consumed = 1;
  let parser = match encoding {
    RAW_ENCODING => {
      let mut parser = ItemParser::new(&pack_name, file_size - consumed);
      while consumed < file_size {
        let n = CHUNK.min(file_size - consumed);
        let bytes = reader.read(n).await?;
        if bytes.len() != n {
          return Err(invalid(&pack_name, format_args!("short RAW read")));
        }
        consumed += bytes.len();
        parser.feed(&bytes)?;
      }
      parser
    }
    LZ4_ENCODING => {
      if file_size - consumed < 4 {
        return Err(invalid(
          &pack_name,
          format_args!("missing LZ4 decoded size"),
        ));
      }
      let size_bytes: [u8; 4] = reader
        .read(4)
        .await?
        .try_into()
        .map_err(|_| invalid(&pack_name, format_args!("invalid LZ4 decoded size")))?;
      consumed += 4;
      let size = u32::from_le_bytes(size_bytes) as usize;
      // Length extensions consume a byte per 255 decoded bytes.
      if size > (file_size - consumed).saturating_mul(255) {
        return Err(invalid(
          &pack_name,
          format_args!("LZ4 decoded size out of bounds"),
        ));
      }
      let mut parser = ItemParser::new(&pack_name, size);
      let first = CHUNK.min(size);
      let window_len = DICT.min(size - first) + first;
      let mut window = Vec::new();
      window.try_reserve_exact(window_len).map_err(|e| {
        invalid(
          &pack_name,
          format_args!("cannot allocate decode window: {e}"),
        )
      })?;
      window.resize(window_len, 0);
      let mut dict_len = 0;
      let mut pos = 0;
      while pos < size {
        if file_size - consumed < 4 {
          return Err(invalid(
            &pack_name,
            format_args!("missing LZ4 chunk length"),
          ));
        }
        let len_bytes: [u8; 4] = reader
          .read(4)
          .await?
          .try_into()
          .map_err(|_| invalid(&pack_name, format_args!("invalid LZ4 chunk length")))?;
        consumed += 4;
        let len = u32::from_le_bytes(len_bytes) as usize;
        let n = CHUNK.min(size - pos);
        if len > file_size - consumed || len > get_maximum_output_size(n) {
          return Err(invalid(
            &pack_name,
            format_args!("LZ4 chunk length out of bounds"),
          ));
        }
        let block = reader.read(len).await?;
        if block.len() != len {
          return Err(invalid(&pack_name, format_args!("short LZ4 read")));
        }
        consumed += len;
        let (dict, out) = window[..dict_len + n].split_at_mut(dict_len);
        let written = decompress_into_with_dict(&block, out, dict)
          .map_err(|e| invalid(&pack_name, format_args!("LZ4 decode failed: {e}")))?;
        if written != n {
          return Err(invalid(
            &pack_name,
            format_args!("LZ4 decoded chunk size mismatch"),
          ));
        }
        parser.feed(out)?;
        let decoded_end = dict_len + n;
        dict_len = DICT.min(decoded_end);
        window.copy_within(decoded_end - dict_len..decoded_end, 0);
        pos += n;
      }
      if consumed != file_size {
        return Err(invalid(
          &pack_name,
          format_args!("trailing bytes after LZ4 chunks"),
        ));
      }
      parser
    }
    _ => return Err(invalid(&pack_name, format_args!("unknown encoding"))),
  };
  reader.close().await?;
  parser.finish()
}
