import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { rspack } from '@rspack/core';
import { closeCompiler } from '@rspack/test-tools/helper/lifecycle';

const initialConsume = 'import lib from "lib"; export default lib;';
const lazyConsume = 'export default () => import("lib");';

function checkContainer(outputPath, startup) {
  execFileSync(process.execPath, [
    '--input-type=module',
    '--eval',
    `
      import assert from 'node:assert/strict';
      import { createRequire } from 'node:module';
      import { pathToFileURL } from 'node:url';

      const [filename, startup] = process.argv.slice(1);
      const require = createRequire(import.meta.url);
      const container = startup === 'async'
        ? await import(pathToFileURL(filename))
        : require(filename);
      if (startup === 'async') {
        assert.throws(() => require(filename), { code: 'ERR_REQUIRE_ASYNC_MODULE' });
      }

      await container.init({});
      const exposed = (await container.get('./exposed'))();
      assert.equal(typeof exposed.then, 'function');
      assert.equal((await exposed).default, 'exposed');
    `,
    path.join(outputPath, 'remoteEntry.mjs'),
    startup,
  ]);
}

function config(context, incremental) {
  const input = context.getDist('input');
  return {
    context: input,
    mode: 'development',
    target: 'node22',
    devtool: false,
    incremental,
    entry: './entry.js',
    output: {
      path: context.getDist(incremental ? 'incremental' : 'clean'),
      filename: '[name].mjs',
      chunkFilename: '[id].mjs',
      module: true,
      uniqueName: 'mf-ordered-startup-rebuild',
    },
    optimization: { runtimeChunk: 'single' },
    resolve: { alias: { lib: path.join(input, 'lib.js') } },
    plugins: [
      new rspack.container.ModuleFederationPlugin({
        name: 'container',
        filename: 'remoteEntry.mjs',
        library: { type: 'module' },
        exposes: { './exposed': './exposed.js' },
      }),
      new rspack.sharing.ConsumeSharedPlugin({
        enhanced: true,
        consumes: {
          lib: {
            shareScope: ['primary', 'default'],
            eager: true,
            requiredVersion: false,
          },
        },
      }),
    ],
  };
}

async function build(compiler, modifiedFile) {
  const stats = await new Promise((resolve, reject) => {
    compiler.run(
      (err, stats) => (err ? reject(err) : resolve(stats)),
      modifiedFile && { modifiedFiles: new Set([modifiedFile]) },
    );
  });
  expect(stats.toJson({ all: false, errors: true }).errors).toEqual([]);
}

export default {
  name: 'container-startup',
  description: 'should stop awaiting container startup after moving the ordered consume out of the initial chunks',
  options(context) {
    const input = context.getDist('input');
    fs.mkdirSync(input, { recursive: true });
    fs.writeFileSync(path.join(input, 'entry.js'), initialConsume);
    fs.writeFileSync(path.join(input, 'lib.js'), 'export default "lib";');
    fs.writeFileSync(
      path.join(input, 'exposed.js'),
      'export default await Promise.resolve("exposed");',
    );
    return config(context, true);
  },
  compiler(_context, compiler) {
    compiler.outputFileSystem = fs;
  },
  async build(context, compiler) {
    const entry = path.join(context.getDist('input'), 'entry.js');
    await build(compiler);
    checkContainer(compiler.outputPath, 'async');

    fs.writeFileSync(entry, lazyConsume);
    await build(compiler, entry);
    const clean = rspack(config(context, false));
    try {
      await build(clean);
      checkContainer(clean.outputPath, 'sync');
      checkContainer(compiler.outputPath, 'sync');
    } finally {
      await closeCompiler(clean);
    }

    fs.writeFileSync(entry, initialConsume);
    await build(compiler, entry);
    checkContainer(compiler.outputPath, 'async');
  },
};
