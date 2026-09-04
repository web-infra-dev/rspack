// Unit tests for the compressed-binding reference packer. Run with:
//   node --test crates/node_binding/scripts/pack-compressed-binding.test.mjs
// Needs a Node with built-in zstd (node:zlib, Node >= 22.15 / 24), the same
// requirement as the packer itself.
import assert from 'node:assert/strict'
import crypto from 'node:crypto'
import { test } from 'node:test'

import { pack, unpack } from './pack-compressed-binding.mjs'

const MAGIC = Buffer.from('__SMOL_PRESSED_DATA_MAGIC_MARKER') // 32B
const HEAD_LEN = MAGIC.length + 8 + 8 + 16 + 3 + 64 + 1 // magic+clen+ulen+key+plat+hash+has_config

// A few representative payloads: tiny, text-ish, and a compressible binary-ish blob.
function payloads() {
  const zeros = Buffer.alloc(4096) // highly compressible
  const text = Buffer.from('rspack '.repeat(2000), 'utf8')
  const mixed = Buffer.concat([
    Buffer.from([0x7f, 0x45, 0x4c, 0x46]), // ELF-ish magic
    crypto.createHash('sha512').update('seed').digest(),
    Buffer.alloc(10000, 0xab),
  ])
  return { zeros, text, mixed }
}

test('pack → unpack round-trips byte-identically', () => {
  for (const [name, raw] of Object.entries(payloads())) {
    const packed = pack(raw)
    const back = unpack(packed)
    assert.ok(back, `${name}: unpack returned null`)
    assert.equal(Buffer.compare(back, raw), 0, `${name}: bytes differ after round-trip`)
  }
})

test('packed output starts with the magic and a full header', () => {
  const packed = pack(Buffer.from('hello'))
  assert.equal(Buffer.compare(packed.subarray(0, MAGIC.length), MAGIC), 0)
  assert.ok(packed.length > HEAD_LEN, 'packed shorter than the header')
})

test('unpack rejects a buffer that is too short', () => {
  assert.equal(unpack(Buffer.alloc(8)), null)
  assert.equal(unpack(Buffer.alloc(HEAD_LEN - 1)), null)
})

test('unpack rejects a wrong magic marker', () => {
  const packed = pack(Buffer.from('payload'))
  packed[0] ^= 0xff // corrupt the first magic byte
  assert.equal(unpack(packed), null)
})

test('unpack rejects a tampered payload (SHA-512 mismatch)', () => {
  const packed = pack(Buffer.from('rspack '.repeat(500)))
  // flip a byte inside the zstd payload (past the header) — hash must catch it
  packed[packed.length - 1] ^= 0x01
  assert.equal(unpack(packed), null)
})

test('unpack rejects an implausibly large uncompressed size', () => {
  const packed = pack(Buffer.from('data'))
  // overwrite the uncompressed-length u64 (offset magic+8) with > 512 MiB cap
  packed.writeBigUInt64LE(BigInt(600 * 1024 * 1024), MAGIC.length + 8)
  assert.equal(unpack(packed), null)
})

test('unpack rejects a compressed length that overruns the buffer', () => {
  const packed = pack(Buffer.from('rspack payload'))
  // clen (offset magic+0) claims more bytes than the section actually holds,
  // but stays under the 512 MiB cap — the fit check must catch it.
  packed.writeBigUInt64LE(BigInt(packed.length + 1024), MAGIC.length)
  assert.equal(unpack(packed), null)
})

test('unpack fails closed (no throw) on a non-zstd payload with a valid hash', () => {
  // Hand-build a well-formed header whose stored SHA-512 matches the payload,
  // but the payload is not a zstd frame — decompress throws, unpack returns null.
  const junk = Buffer.from('this is not a zstd frame', 'utf8')
  const head = Buffer.alloc(HEAD_LEN)
  let o = 0
  MAGIC.copy(head, o)
  o += MAGIC.length
  head.writeBigUInt64LE(BigInt(junk.length), o) // clen
  o += 8
  head.writeBigUInt64LE(64n, o) // ulen (plausible, under cap)
  o += 8
  o += 16 + 3 // cache key + platform
  crypto.createHash('sha512').update(junk).digest().copy(head, o) // matching hash
  o += 64
  head[o] = 0 // has_config
  const section = Buffer.concat([head, junk])
  assert.equal(unpack(section), null)
})
