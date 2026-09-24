// Pack a built `rspack.<platform>.node` into the "pressed-data" payload used by
// the compressed-binding loader: zstd-compressed addon + SHA-512 integrity, in
// the same layout the `decmpfs` crate (and bin-infra) already unwraps. This is
// the creator-side reference; the build step injects the payload into the
// binary's PRESSED_DATA section and the Rust loader unwraps it on first load.
//
// Format: [magic 32B "__SMOL_PRESSED_DATA_MAGIC_MARKER"]
//   [compressed u64 LE][uncompressed u64 LE][cache key 16B][platform 3B]
//   [SHA-512 of the zstd payload 64B][has_config 1B][zstd frame]
//
// No @napi-rs/cli, no legacy-Node fallback — modern node:zlib/node:crypto only.
import crypto from 'node:crypto'
import zlib from 'node:zlib'

const MAGIC = Buffer.from('__SMOL_PRESSED_DATA_MAGIC_MARKER') // 32B
const CACHE_KEY_LEN = 16
const PLATFORM_LEN = 3
const HASH_LEN = 64 // SHA-512

export function pack(raw, options) {
  const level = options?.level ?? 19
  const payload = zlib.zstdCompressSync(raw, {
    [zlib.constants.ZSTD_c_compressionLevel]: level,
  })
  const hash = crypto.createHash('sha512').update(payload).digest()
  const head = Buffer.alloc(
    MAGIC.length + 8 + 8 + CACHE_KEY_LEN + PLATFORM_LEN + HASH_LEN + 1,
  )
  let o = 0
  MAGIC.copy(head, o)
  o += MAGIC.length
  head.writeBigUInt64LE(BigInt(payload.length), o)
  o += 8
  head.writeBigUInt64LE(BigInt(raw.length), o)
  o += 8
  crypto.createHash('sha256').update(raw).digest().copy(head, o, 0, CACHE_KEY_LEN)
  o += CACHE_KEY_LEN
  o += PLATFORM_LEN // platform/arch/libc — stamped by the build matrix
  hash.copy(head, o)
  o += HASH_LEN
  head[o] = 0 // has_config
  return Buffer.concat([head, payload])
}

const MAX_LEN = 512 * 1024 * 1024 // reject implausible sizes; keeps sizes in Number's safe range

export function unpack(section) {
  const min = MAGIC.length + 8 + 8 + CACHE_KEY_LEN + PLATFORM_LEN + HASH_LEN + 1
  if (section.length < min || !section.subarray(0, MAGIC.length).equals(MAGIC)) {
    return null
  }
  let o = MAGIC.length
  const clenBig = section.readBigUInt64LE(o)
  o += 8
  const ulenBig = section.readBigUInt64LE(o)
  o += 8
  o += CACHE_KEY_LEN + PLATFORM_LEN
  const hash = section.subarray(o, o + HASH_LEN)
  o += HASH_LEN
  if (section[o]) {
    o += 1192 // config block
  }
  o += 1
  // Bound both lengths BEFORE narrowing to Number, so a huge u64 can't lose
  // precision or drive an out-of-range read.
  if (
    clenBig <= 0n ||
    ulenBig <= 0n ||
    clenBig > BigInt(MAX_LEN) ||
    ulenBig > BigInt(MAX_LEN)
  ) {
    return null
  }
  const clen = Number(clenBig)
  const ulen = Number(ulenBig)
  // The declared payload must actually fit in what's left of the section.
  if (o + clen > section.length) {
    return null
  }
  const payload = section.subarray(o, o + clen)
  if (!crypto.createHash('sha512').update(payload).digest().equals(hash)) {
    return null
  }
  // A corrupt payload that still matched the hash (or a non-zstd frame) must
  // fail closed, never crash the loader.
  let raw
  try {
    raw = zlib.zstdDecompressSync(payload)
  } catch {
    return null
  }
  return raw.length === ulen ? raw : null
}
