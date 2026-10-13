import { $ } from 'zx';
import { constants, copyFileSync, readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

const root = path.resolve(import.meta.dirname, '..');

/** Build the host compiler driver before starting an allocative-enabled Cargo invocation. */
export async function prepareAllocativeDriver(environment = process.env) {
  const toolchain = readFileSync(
    path.join(root, 'rust-toolchain.toml'),
    'utf8',
  ).match(/^channel\s*=\s*"([^"]+)"/m)?.[1];
  if (!toolchain) throw new Error('Missing pinned Rust toolchain');
  if (environment.RUSTC_WORKSPACE_WRAPPER) {
    throw new Error(
      'Allocative needs RUSTC_WORKSPACE_WRAPPER; unset the existing workspace wrapper',
    );
  }
  const buildEnv = {
    ...environment,
    RUSTUP_TOOLCHAIN: toolchain,
    RUSTC_WRAPPER: '',
    RUSTC_WORKSPACE_WRAPPER: '',
    RUSTFLAGS: '',
    CARGO_ENCODED_RUSTFLAGS: '',
  };
  const run = $({ cwd: root, env: buildEnv, verbose: false });
  await run`rustup component add rustc-dev --toolchain ${toolchain}`;
  const version = await run`rustc -vV`;
  const host = version.stdout.match(/^host: (.+)$/m)?.[1];
  if (!host) throw new Error('rustc did not report its host');
  const sysroot = (await run`rustc --print sysroot`).stdout.trim();
  const flags = ['-Crpath', `-Lnative=${path.join(sysroot, 'lib')}`];
  const compile = $({
    cwd: root,
    env: { ...buildEnv, CARGO_ENCODED_RUSTFLAGS: flags.join('\x1f') },
    verbose: false,
  });
  const build =
    await compile`cargo build -p rspack_allocative_driver --features driver --profile allocative-driver --target ${host} --message-format=json-render-diagnostics`;
  const artifacts = build.stdout
    .trim()
    .split('\n')
    .map((line) => JSON.parse(line));
  const executable = artifacts.find(
    (item) =>
      item.reason === 'compiler-artifact' &&
      item.target.name === 'rspack-allocative-driver' &&
      item.executable,
  )?.executable;
  if (!executable)
    throw new Error('Cargo did not produce the allocative compiler driver');
  // Cargo separates workspace artifacts by wrapper path. Include the driver contents so
  // changing instrumentation cannot reuse artifacts produced by an older driver.
  const hash = createHash('sha256')
    .update(readFileSync(executable))
    .digest('hex');
  const extension = process.platform === 'win32' ? '.exe' : '';
  const wrapper = path.join(
    path.dirname(executable),
    `rspack-allocative-driver-${hash}${extension}`,
  );
  try {
    copyFileSync(executable, wrapper, constants.COPYFILE_EXCL);
  } catch (error) {
    if (error.code !== 'EEXIST') throw error;
  }
  return {
    ...environment,
    RUSTUP_TOOLCHAIN: toolchain,
    RUSTC_WORKSPACE_WRAPPER: wrapper,
    RSPACK_ALLOCATIVE_DRIVER: '1',
  };
}

const scriptIndex = process.argv.findIndex(
  (argument) => pathToFileURL(path.resolve(argument)).href === import.meta.url,
);
if (scriptIndex > 0) {
  const command = process.argv.slice(scriptIndex + 1);
  if (!command.length) {
    throw new Error(
      'Usage: zx scripts/allocative-driver.mjs cargo <arguments>',
    );
  }
  const env = await prepareAllocativeDriver();
  await $({
    cwd: process.cwd(),
    env,
    stdio: 'inherit',
    verbose: false,
  })`${command}`;
}
