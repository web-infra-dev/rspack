/* eslint-disable */
// @ts-nocheck

const { existsSync, readFileSync } = require('node:fs');
const path = require('node:path');

const RUNTIME_PATH_ENV = 'RSPACK_WASM_RUNTIME_LIBRARY_PATH';
const loadErrors = [];

const isMusl = () => {
  let musl = false;
  if (process.platform === 'linux') {
    musl = isMuslFromFilesystem();
    if (musl === null) {
      musl = isMuslFromReport();
    }
    if (musl === null) {
      musl = isMuslFromChildProcess();
    }
  }
  return musl;
};

const isFileMusl = (f) => f.includes('libc.musl-') || f.includes('ld-musl-');

const isMuslFromFilesystem = () => {
  try {
    return readFileSync('/usr/bin/ldd', 'utf-8').includes('musl');
  } catch {
    return null;
  }
};

const isMuslFromReport = () => {
  let report = null;
  if (typeof process.report?.getReport === 'function') {
    process.report.excludeNetwork = true;
    report = process.report.getReport();
  }
  if (!report) {
    return null;
  }
  if (report.header && report.header.glibcVersionRuntime) {
    return false;
  }
  if (Array.isArray(report.sharedObjects)) {
    if (report.sharedObjects.some(isFileMusl)) {
      return true;
    }
  }
  return false;
};

const isMuslFromChildProcess = () => {
  try {
    return require('node:child_process')
      .execSync('ldd --version', { encoding: 'utf8' })
      .includes('musl');
  } catch {
    return false;
  }
};

function getPlatformArchABI() {
  if (process.platform === 'darwin') {
    if (process.arch === 'x64') return 'darwin-x64';
    if (process.arch === 'arm64') return 'darwin-arm64';
  }

  if (process.platform === 'win32') {
    if (process.arch === 'x64') return 'win32-x64-msvc';
  }

  if (process.platform === 'linux') {
    if (process.arch === 'x64') {
      return isMusl() ? 'linux-x64-musl' : 'linux-x64-gnu';
    }
    if (process.arch === 'arm64') {
      return isMusl() ? 'linux-arm64-musl' : 'linux-arm64-gnu';
    }
  }

  throw new Error(
    `Unsupported platform for @rspack/wasm-runtime: ${process.platform} ${process.arch}`,
  );
}

function getDynamicLibraryExtension() {
  if (process.platform === 'win32') return 'dll';
  if (process.platform === 'darwin') return 'dylib';
  return 'so';
}

function getDynamicLibraryName(platformArchABI = getPlatformArchABI()) {
  return `rspack_wasm_runtime.${platformArchABI}.${getDynamicLibraryExtension()}`;
}

function resolveWasmRuntime() {
  if (process.env[RUNTIME_PATH_ENV]) {
    return process.env[RUNTIME_PATH_ENV];
  }

  const platformArchABI = getPlatformArchABI();
  const localPath = path.join(
    __dirname,
    getDynamicLibraryName(platformArchABI),
  );
  if (existsSync(localPath)) {
    return localPath;
  }

  const packageName = `@rspack/wasm-runtime-${platformArchABI}`;
  try {
    return require.resolve(packageName);
  } catch (error) {
    loadErrors.push(error);
  }

  throw new Error(
    `Cannot find ${packageName}. Please reinstall @rspack/wasm-runtime or set ${RUNTIME_PATH_ENV}.` +
      `\n\n${loadErrors.map((e) => e.message).join('\n')}`,
    { cause: loadErrors },
  );
}

module.exports.resolveWasmRuntime = resolveWasmRuntime;
