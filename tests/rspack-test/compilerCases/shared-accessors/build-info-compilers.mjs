import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { rspack } from '@rspack/core';
import { createFsFromVolume, Volume } from 'memfs';
import {
  closeCompiler,
  runCompiler,
} from '@rspack/test-tools/helper/lifecycle';

export default async function run() {
  const context = fs.realpathSync(
    fs.mkdtempSync(path.join(os.tmpdir(), 'rspack-build-info-compilers-')),
  );
  const compilers = [];
  try {
    const entryPath = path.join(context, 'index.js');
    const dependencies = ['first.txt', 'second.txt'].map((name) =>
      path.join(context, name),
    );
    fs.writeFileSync(entryPath, 'export default 42;');
    for (const dependency of dependencies) fs.writeFileSync(dependency, '');
    const modules = [];
    const infos = [];
    const fileKey = Symbol.for('rspack.buildInfo.fileDependencies');
    for (const [index, dependency] of dependencies.entries()) {
      const compiler = rspack({
        context,
        entry: './index.js',
        mode: 'none',
        cache: false,
        module: {
          rules: [
            {
              test: /\.js$/,
              use: [
                {
                  loader: fileURLToPath(
                    new URL('./build-info-loader.cjs', import.meta.url),
                  ),
                  options: { dependency },
                },
              ],
            },
          ],
        },
        output: { path: path.join(context, `dist-${index}`) },
        plugins: [
          (compiler) => {
            compiler.hooks.done.tap('BuildInfoCompilers', ({ compilation }) => {
              const module = [...compilation.modules].find(
                (module) => module.resource === entryPath,
              );
              assert(module);
              modules[index] = module;
              infos[index] = module.buildInfo;
            });
          },
        ],
      });
      compilers.push(compiler);
      compiler.outputFileSystem = createFsFromVolume(new Volume());
      const stats = await runCompiler(compiler);
      assert.equal(stats.hasErrors(), false);
    }
    assert.equal(modules[0].identifier(), modules[1].identifier());
    const getFiles = Object.getOwnPropertyDescriptor(infos[0], fileKey).get;
    for (const index of [0, 1]) {
      const files = getFiles.call(infos[index]);
      assert(files.includes(dependencies[index]));
      assert(!files.includes(dependencies[1 - index]));
    }
  } finally {
    await Promise.all(compilers.map(closeCompiler));
    fs.rmSync(context, { recursive: true, force: true });
  }
}
