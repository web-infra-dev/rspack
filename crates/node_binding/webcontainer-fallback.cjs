const fs = require('node:fs');
const path = require('node:path');
const childProcess = require('node:child_process');

const pkg = JSON.parse(
  fs.readFileSync(path.join(__dirname, 'package.json'), 'utf-8'),
);
const version = pkg.version;
const baseDir = `/tmp/rspack-${version}`;
const bindingEntry = `${baseDir}/node_modules/${pkg.name}-wasm32-wasi/rspack.wasi.cjs`;

if (!fs.existsSync(bindingEntry)) {
  fs.rmSync(baseDir, { recursive: true, force: true });
  fs.mkdirSync(baseDir, { recursive: true });
  const bindingPkg = `${pkg.name}-wasm32-wasi@${version}`;
  console.log(`[rspack] Downloading ${bindingPkg} on WebContainer...`);
  childProcess.execFileSync('pnpm', ['i', bindingPkg], {
    cwd: baseDir,
    stdio: 'inherit',
  });
}

module.exports = require(bindingEntry);