// @ts-nocheck
const path = require('node:path');
const fs = require('node:fs');
const assert = require('node:assert');

// Generates binding packages based on artifacts.
// Note: it's dedicated to work with pnpm workspaces.

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

const AbiToNodeLibc = {
  gnu: 'glibc',
  musl: 'musl',
};

/**
 * A triple is a specific format for specifying a target architecture.
 * Triples may be referred to as a target triple which is the architecture for the artifact produced, and the host triple which is the architecture that the compiler is running on.
 * The general format of the triple is `<arch><sub>-<vendor>-<sys>-<abi>` where:
 *   - `arch` = The base CPU architecture, for example `x86_64`, `i686`, `arm`, `thumb`, `mips`, etc.
 *   - `sub` = The CPU sub-architecture, for example `arm` has `v7`, `v7s`, `v5te`, etc.
 *   - `vendor` = The vendor, for example `unknown`, `apple`, `pc`, `nvidia`, etc.
 *   - `sys` = The system name, for example `linux`, `windows`, `darwin`, etc. none is typically used for bare-metal without an OS.
 *   - `abi` = The ABI, for example `gnu`, `android`, `eabi`, etc.
 */
function parseTriple(rawTriple) {
  const triple = rawTriple.endsWith('eabi')
    ? `${rawTriple.slice(0, -4)}-eabi`
    : rawTriple;
  const triples = triple.split('-');
  let cpu;
  let sys;
  let abi = null;
  if (triples.length === 4) {
    [cpu, , sys, abi = null] = triples;
  } else if (triples.length === 3) {
    [cpu, , sys] = triples;
  } else {
    [cpu, sys] = triples;
  }
  const platformName = SysToNodePlatform[sys] ?? sys;
  const arch = CpuToNodeArch[cpu] ?? cpu;
  return {
    platform: platformName,
    arch,
    abi,
    platformArchABI: abi
      ? `${platformName}-${arch}-${abi}`
      : `${platformName}-${arch}`,
    raw: rawTriple,
  };
}

function generateReadme(pkgName) {
  const README = `<picture>
  <img alt="Rspack Banner" src="https://assets.rspack.rs/rspack/rspack-banner.png">
</picture>

# ${pkgName}

Private node binding crate for rspack.

This package does *NOT* follow [semantic versioning](https://semver.org/).

## Documentation

See [https://rspack.rs](https://rspack.rs) for details.

## License

Rspack is [MIT licensed](https://github.com/web-infra-dev/rspack/blob/main/LICENSE).
`;
  return README;
}

const ARTIFACTS = path.resolve(__dirname, '../artifacts');
const NPM = path.resolve(__dirname, '../npm');

try {
  // Error tolerant if the directory already exists
  fs.mkdirSync(NPM);
} catch {}

// Releasing bindings
const releasingPackages = [];

const bindings = fs
  .readdirSync(ARTIFACTS, {
    withFileTypes: true,
  })
  .filter((item) => item.isDirectory())
  .map((item) => path.join(ARTIFACTS, item.name));

const optionalDependencies = {};

for (const binding of bindings) {
  // The pkg of wasm binding is more complex, so we create it manually.
  if (binding.includes('wasm')) {
    const output = path.join(NPM, 'wasm32-wasi');
    const pkgJson = require(path.join(output, 'package.json'));

    optionalDependencies[pkgJson.name] = 'workspace:*';

    // Copy wasm artifact
    fs.copyFileSync(
      path.join(binding, 'rspack.wasm32-wasi.wasm'),
      path.join(output, 'rspack.wasm32-wasi.wasm'),
    );

    // Copy wasm js runtimes from the node_binding crate
    const NODE_BINDING_CRATE = path.resolve(
      __dirname,
      '../crates/node_binding/',
    );
    for (const file of [
      'rspack.wasi.cjs',
      'rspack.wasi-browser.js',
      'wasi-worker.mjs',
      'wasi-worker-browser.mjs',
    ]) {
      fs.copyFileSync(
        path.join(NODE_BINDING_CRATE, file),
        path.join(output, file),
      );
    }

    const README = generateReadme(pkgJson.name);
    fs.writeFileSync(path.join(output, 'README.md'), README);

    releasingPackages.push(pkgJson.name);
    continue;
  }

  // bindings-x86_64-unknown-linux-musl
  const files = fs.readdirSync(binding);
  assert(files.length === 1, `Expected only one file in ${binding}`);

  // rspack.linux-x64-musl.node
  const file = files[0];
  assert(path.extname(file) === '.node', `Expected .node file in ${binding}`);
  const binary = fs.readFileSync(path.join(binding, file));

  const name = path.basename(binding);
  assert(name.startsWith('bindings-'));

  // x86_64-unknown-linux-musl
  const {
    // linux, darwin, win32
    platform,
    // x64, arm64
    arch,
    // musl, gnu, null
    abi,
    // linux-x64-musl
    platformArchABI,
  } = parseTriple(name.slice(9));
  assert(
    file.split('.')[1] === platformArchABI,
    `Binding is not matched with triple (expected: rspack.${platformArchABI}.node, got: ${file})`,
  );

  // <absolute-path-to-npm>/linux-x64-musl
  const output = path.join(NPM, platformArchABI);
  try {
    fs.mkdirSync(output);
  } catch {}

  const coreJson = require(
    path.resolve(__dirname, '../packages/rspack/package.json'),
  );
  const pkgJson = {};
  pkgJson.name = `@rspack/binding-${platformArchABI}`;
  pkgJson.version = coreJson.version;
  pkgJson.license = coreJson.license;
  pkgJson.description = 'Node binding for rspack';
  pkgJson.main = `rspack.${platformArchABI}.node`;
  pkgJson.homepage = coreJson.homepage;
  pkgJson.bugs = coreJson.bugs;
  pkgJson.repository = coreJson.repository;
  pkgJson.publishConfig = {
    access: 'public',
  };
  pkgJson.files = [pkgJson.main];
  pkgJson.os = [platform];
  pkgJson.cpu = [arch];
  if (abi && AbiToNodeLibc[abi]) {
    pkgJson.libc = [AbiToNodeLibc[abi]];
  }

  // Using pnpm workspace
  optionalDependencies[pkgJson.name] = 'workspace:*';

  fs.writeFileSync(`${output}/package.json`, JSON.stringify(pkgJson, null, 2));
  fs.writeFileSync(`${output}/${pkgJson.main}`, binary);

  const README = generateReadme(pkgJson.name);

  fs.writeFileSync(`${output}/README.md`, README);
  releasingPackages.push(pkgJson.name);
}

// Determine whether to release or not based on the CI build result.
// Validating not releasable bindings
const dirent = fs.readdirSync(NPM, {
  withFileTypes: true,
});
for (const item of dirent) {
  if (item.isDirectory()) {
    const dir = path.join(NPM, item.name);
    const pkg = require(`${dir}/package.json`);

    if (releasingPackages.includes(pkg.name)) {
      // releasing
      console.info(`Releasing package: ${pkg.name}`);
    } else {
      pkg.private = true;
      console.info(
        `Skipping package: ${pkg.name}. (Reason: local package, but its artifact is not available.)`,
      );
      fs.writeFileSync(`${dir}/package.json`, JSON.stringify(pkg, null, 2));
    }
  }
}

const bindingJsonPath = path.resolve(
  __dirname,
  '../crates/node_binding/package.json',
);
const bindingJson = require(bindingJsonPath);

// The original `optionalDependencies` field in `package.json` is used to publish locally, so we have to override it for CI.
bindingJson.optionalDependencies = optionalDependencies;

fs.writeFileSync(bindingJsonPath, JSON.stringify(bindingJson, null, 2));
