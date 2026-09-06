// Report what the compressed binding buys for THIS machine's installed
// `@rspack/binding`: the shipped `.node` download size, raw vs zstd-19. Resolves
// whichever version is installed for the current platform — no hardcoded
// version or path.
//
// Load speed comes from the OS's transparent filesystem compression (APFS /
// NTFS / btrfs) that the loader applies on first use: on such a filesystem the
// addon takes fewer blocks on disk AND loads faster, because the kernel reads
// fewer bytes and decompresses them cheaply. That measurement needs a
// compressing filesystem and is reported in the PR.
//
// Run: `node crates/node_binding/scripts/bench-compressed-binding.mjs`
// (from a checkout where `@rspack/binding-<triple>` is installed).
import crypto from 'node:crypto'
import { createRequire } from 'node:module'
import fs from 'node:fs'
import path from 'node:path'
import process from 'node:process'
import { fileURLToPath } from 'node:url'
import zlib from 'node:zlib'

const require = createRequire(import.meta.url)

// napi-rs platform triple for the `@rspack/binding-<triple>` packages. macOS
// and Windows have no libc suffix; Linux is glibc unless a musl libc reports.
function platformTriple() {
  const { arch, platform } = process
  if (platform === 'linux') {
    const isMusl =
      typeof process.report?.getReport === 'function' &&
      /musl/.test(JSON.stringify(process.report.getReport().header ?? {}))
    return `linux-${arch}-${isMusl ? 'musl' : 'gnu'}`
  }
  if (platform === 'win32') {
    return `win32-${arch}-msvc`
  }
  return `${platform}-${arch}`
}

// Resolve the installed binding `.node` for this platform. Try node's own
// resolution first; fall back to scanning `node_modules/@rspack` so a pnpm
// symlink layout or an odd cwd still finds it.
function resolveBindingNode(triple) {
  const file = `rspack.${triple}.node`
  try {
    return require.resolve(`@rspack/binding-${triple}/${file}`)
  } catch {}
  const roots = [process.cwd(), path.dirname(fileURLToPath(import.meta.url))]
  for (const root of roots) {
    let dir = root
    for (let i = 0; i < 8; i += 1) {
      const hit = path.join(
        dir,
        'node_modules',
        '@rspack',
        `binding-${triple}`,
        file,
      )
      if (fs.existsSync(hit)) {
        return hit
      }
      const parent = path.dirname(dir)
      if (parent === dir) {
        break
      }
      dir = parent
    }
  }
  throw new Error(
    `no installed @rspack/binding-${triple} found — install it, or run from a checkout that has it`,
  )
}

function mib(bytes) {
  return `${(bytes / 1024 / 1024).toFixed(1)} MiB`
}

const triple = platformTriple()
const src = resolveBindingNode(triple)
const raw = fs.readFileSync(src)

const t0 = process.hrtime.bigint()
const zst = zlib.zstdCompressSync(raw, {
  [zlib.constants.ZSTD_c_compressionLevel]: 19,
})
const t1 = process.hrtime.bigint()
const savedPct = (100 - (zst.length / raw.length) * 100).toFixed(0)
const sha = crypto.createHash('sha512').update(zst).digest('hex').slice(0, 12)

console.log(`platform   ${triple}`)
console.log(`binding    ${src}`)
console.log(`raw        ${mib(raw.length)} (${raw.length} bytes)`)
console.log(
  `zstd-19    ${mib(zst.length)}  (−${savedPct}% download, sha512:${sha}, ${(Number(t1 - t0) / 1e6).toFixed(0)} ms to compress)`,
)
