/**
 * From https://github.com/oxc-project/oxc/blob/main/napi/parser/webcontainer-fallback.js
 */

const fs = require('node:fs');
const childProcess = require('node:child_process');

const pkg = JSON.parse(
  fs.readFileSync(require.resolve('@rspack/binding/package.json'), 'utf-8'),
);
const version = pkg.version;
const baseDir = `/tmp/rspack-${version}`;
const bindingEntry = `${baseDir}/node_modules/@rspack/binding-wasm32-wasi/rspack.wasi.cjs`;

if (!fs.existsSync(bindingEntry)) {
  fs.rmSync(baseDir, { recursive: true, force: true });
  fs.mkdirSync(baseDir, { recursive: true });
  const bindingPkg = `@rspack/binding-wasm32-wasi@${version}`;
  console.log(`[rspack] Downloading ${bindingPkg} on WebContainer...`);
  childProcess.execFileSync('pnpm', ['i', bindingPkg], {
    cwd: baseDir,
    stdio: 'inherit',
  });
}

module.exports = require(bindingEntry);