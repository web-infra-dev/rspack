const path = require('node:path');
const fs = require('node:fs');
const { spawnSync } = require('node:child_process');
const { values } = require('node:util').parseArgs({
  args: process.argv.slice(2),
  options: {
    profile: {
      type: 'string',
    },
  },
  strict: true,
});

const CpuToNodeArch = {
  x86_64: 'x64',
  aarch64: 'arm64',
  i686: 'ia32',
  armv7: 'arm',
};

const SysToNodePlatform = {
  linux: 'linux',
  freebsd: 'freebsd',
  darwin: 'darwin',
  windows: 'win32',
};

function parseTriple(rawTriple) {
  const triple = rawTriple.endsWith('eabi')
    ? `${rawTriple.slice(0, -4)}-eabi`
    : rawTriple;
  const triples = triple.split('-');
  let cpu;
  let sys;
  let abi = null;
  if (triples.length === 4) {
    cpu = triples[0];
    sys = triples[2];
    abi = triples[3] ?? null;
  } else if (triples.length === 3) {
    cpu = triples[0];
    sys = triples[2];
  } else {
    [cpu, sys] = triples;
  }
  const platform = SysToNodePlatform[sys] ?? sys;
  const arch = CpuToNodeArch[cpu] ?? cpu;
  return {
    platform,
    arch,
    abi,
    platformArchABI: abi ? `${platform}-${arch}-${abi}` : `${platform}-${arch}`,
  };
}

function currentTriple() {
  const arch = process.arch === 'x64' ? 'x86_64' : 'aarch64';
  if (process.platform === 'darwin') return `${arch}-apple-darwin`;
  if (process.platform === 'linux') return `${arch}-unknown-linux-gnu`;
  if (process.platform === 'win32') return `${arch}-pc-windows-msvc`;
  throw new Error(`Unsupported platform: ${process.platform} ${process.arch}`);
}

function dynamicLibraryExtension(platform) {
  if (platform === 'win32') return 'dll';
  if (platform === 'darwin') return 'dylib';
  return 'so';
}

function sourceLibraryName(platform) {
  if (platform === 'win32') return 'rspack_wasm_runtime.dll';
  if (platform === 'darwin') return 'librspack_wasm_runtime.dylib';
  return 'librspack_wasm_runtime.so';
}

if (
  process.env.DISABLE_PLUGIN ||
  process.env.RSPACK_TARGET_BROWSER ||
  process.env.RUST_TARGET?.startsWith('wasm32')
) {
  process.exit(0);
}

const root = path.resolve(__dirname, '../../..');
const target = process.env.RUST_TARGET || currentTriple();
const hasExplicitTarget = Boolean(process.env.RUST_TARGET);
const { platform, platformArchABI } = parseTriple(target);
const profile = values.profile || 'debug';
const profileDir = profile === 'dev' ? 'debug' : profile;
const args = ['build', '-p', 'rspack_wasm_runtime'];

if (values.profile) {
  args.push('--profile', values.profile);
}

if (hasExplicitTarget) {
  args.push('--target', process.env.RUST_TARGET);
}

console.log(`Run command: cargo ${args.join(' ')}`);
const result = spawnSync('cargo', args, {
  cwd: root,
  stdio: 'inherit',
  env: process.env,
});

if (result.status !== 0) {
  process.exit(result.status ?? 1);
}

const source = path.join(
  root,
  'target',
  ...(hasExplicitTarget ? [target] : []),
  profileDir,
  sourceLibraryName(platform),
);
const output = path.resolve(
  __dirname,
  '..',
  `rspack_wasm_runtime.${platformArchABI}.${dynamicLibraryExtension(platform)}`,
);

fs.copyFileSync(source, output);
