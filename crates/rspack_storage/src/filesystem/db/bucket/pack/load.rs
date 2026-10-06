use std::hash::{Hash, Hasher};

use lz4_flex::block::{decompress_into, decompress_into_with_dict, get_maximum_output_size};
use rustc_hash::FxHasher;

use super::{CHUNK, DICT, LZ4_ENCODING, Pack, PackId, RAW_ENCODING, ScopeFileSystem};
use crate::{Error, Result};

// The "{key_len} {value_len}" header needs at most 41 bytes on 64-bit platforms.
// Leave room for leading zeroes without allowing corrupt packs to grow an unbounded header.
const MAX_HEADER: usize = 128;

#[cold]
#[inline(never)]
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

  #[inline(never)]
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

// Framing and decoding stay synchronous so RAW and LZ4 share one read await
// and one copy of the parser, codec and invalid-format paths. Both `feed`
// methods are `#[inline(never)]` to keep that single copy and the binary small.
enum Input {
  Encoding,
  Raw,
  Size,
  Length,
  Block(usize),
}

struct Decoder<'a> {
  input: Input,
  file_remaining: usize,
  decoded_remaining: usize,
  dict_len: usize,
  window: Vec<u8>,
  parser: ItemParser<'a>,
}

impl<'a> Decoder<'a> {
  fn new(pack_name: &'a str, file_size: usize) -> Self {
    Self {
      input: Input::Encoding,
      file_remaining: file_size,
      decoded_remaining: 0,
      dict_len: 0,
      window: Vec::new(),
      parser: ItemParser::new(pack_name, 0),
    }
  }

  fn read_len(&self) -> Result<Option<usize>> {
    let pack_name = self.parser.pack_name;
    match self.input {
      Input::Encoding => Ok(Some(1)),
      Input::Raw => Ok((self.file_remaining != 0).then(|| CHUNK.min(self.file_remaining))),
      Input::Size => {
        if self.file_remaining < 4 {
          return Err(invalid(pack_name, "missing LZ4 decoded size"));
        }
        Ok(Some(4))
      }
      Input::Length => {
        if self.decoded_remaining == 0 {
          if self.file_remaining != 0 {
            return Err(invalid(pack_name, "trailing bytes after LZ4 chunks"));
          }
          return Ok(None);
        }
        if self.file_remaining < 4 {
          return Err(invalid(pack_name, "missing LZ4 chunk length"));
        }
        Ok(Some(4))
      }
      Input::Block(len) => Ok(Some(len)),
    }
  }

  /// `bytes` must be exactly the length returned by the preceding `read_len`;
  /// the remaining-length arithmetic below relies on it.
  #[inline(never)]
  fn feed(&mut self, bytes: &[u8]) -> Result<()> {
    let pack_name = self.parser.pack_name;
    match self.input {
      Input::Encoding => {
        let encoding = *bytes
          .first()
          .ok_or_else(|| invalid(pack_name, "missing encoding"))?;
        self.file_remaining -= 1;
        self.input = match encoding {
          RAW_ENCODING => {
            self.parser.remaining = self.file_remaining;
            Input::Raw
          }
          LZ4_ENCODING => Input::Size,
          _ => return Err(invalid(pack_name, "unknown encoding")),
        };
      }
      Input::Raw => {
        if bytes.len() != CHUNK.min(self.file_remaining) {
          return Err(invalid(pack_name, "short RAW read"));
        }
        self.file_remaining -= bytes.len();
        self.parser.feed(bytes)?;
      }
      Input::Size => {
        let size_bytes = bytes
          .try_into()
          .map_err(|_| invalid(pack_name, "invalid LZ4 decoded size"))?;
        self.file_remaining -= 4;
        let size = u32::from_le_bytes(size_bytes) as usize;
        // Length extensions consume a byte per 255 decoded bytes.
        if size > self.file_remaining.saturating_mul(255) {
          return Err(invalid(pack_name, "LZ4 decoded size out of bounds"));
        }
        self.decoded_remaining = size;
        self.parser.remaining = size;
        let first = CHUNK.min(size);
        let window_len = DICT.min(size - first) + first;
        self
          .window
          .try_reserve_exact(window_len)
          .map_err(|_| invalid(pack_name, "cannot allocate decode window"))?;
        self.window.resize(window_len, 0);
        self.input = Input::Length;
      }
      Input::Length => {
        let len_bytes = bytes
          .try_into()
          .map_err(|_| invalid(pack_name, "invalid LZ4 chunk length"))?;
        self.file_remaining -= 4;
        let len = u32::from_le_bytes(len_bytes) as usize;
        let n = CHUNK.min(self.decoded_remaining);
        if len > self.file_remaining || len > get_maximum_output_size(n) {
          return Err(invalid(pack_name, "LZ4 chunk length out of bounds"));
        }
        self.input = Input::Block(len);
      }
      Input::Block(len) => {
        if bytes.len() != len {
          return Err(invalid(pack_name, "short LZ4 read"));
        }
        self.file_remaining -= len;
        let n = CHUNK.min(self.decoded_remaining);
        let decoded_end = self.dict_len + n;
        let (dict, out) = self.window[..decoded_end].split_at_mut(self.dict_len);
        let written = if dict.is_empty() {
          decompress_into(bytes, out)
        } else {
          decompress_into_with_dict(bytes, out, dict)
        }
        .map_err(|_| invalid(pack_name, "LZ4 decode failed"))?;
        if written != n {
          return Err(invalid(pack_name, "LZ4 decoded chunk size mismatch"));
        }
        self.parser.feed(out)?;
        self.dict_len = DICT.min(decoded_end);
        self
          .window
          .copy_within(decoded_end - self.dict_len..decoded_end, 0);
        self.decoded_remaining -= n;
        self.input = Input::Length;
      }
    }
    Ok(())
  }
}

pub(super) async fn load(fs: &ScopeFileSystem, id: PackId) -> Result<(Pack, u64)> {
  let pack_name = id.pack_name();
  let file_size = fs.stat(&pack_name).await?.size;
  if file_size == 0 {
    return Err(invalid(&pack_name, "encoded size out of bounds"));
  }
  let file_size =
    usize::try_from(file_size).map_err(|_| invalid(&pack_name, "encoded size out of bounds"))?;
  let mut reader = fs.stream_read(&pack_name).await?;
  let mut decoder = Decoder::new(&pack_name, file_size);
  while let Some(len) = decoder.read_len()? {
    let bytes = reader.read(len).await?;
    decoder.feed(&bytes)?;
  }
  let parser = decoder.parser;
  // Free the decode window before awaiting close; only the parsed items outlive the stream.
  drop(decoder.window);
  reader.close().await?;
  parser.finish()
}
