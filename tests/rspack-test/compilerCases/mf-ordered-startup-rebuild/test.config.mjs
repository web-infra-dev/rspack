import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { rspack } from '@rspack/core';
import { closeCompiler } from '@rspack/test-tools/helper/lifecycle';

const initialConsume = 'import lib from "lib"; export const answer = lib;';
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

function checkLibrary(outputPath, load = 'import') {
  execFileSync(process.execPath, [
    '--input-type=module',
    '--eval',
    `
      import assert from 'node:assert/strict';
      import { createRequire } from 'node:module';
      import { pathToFileURL } from 'node:url';

      const [filename, load] = process.argv.slice(1);
      const library = load === 'require'
        ? createRequire(import.meta.url)(filename)
        : await import(pathToFileURL(filename));
      assert.equal(library.answer, 42);
    `,
    path.join(outputPath, 'library.mjs'),
    load,
  ]);
}

function config(context, incremental, inlineRuntime = false) {
  const input = context.getDist('input');
  const library = {
    import: inlineRuntime ? './entry.js' : './library.js',
    library: { type: 'module' },
  };
  return {
    context: input,
    mode: 'development',
    target: 'node22',
    devtool: false,
    incremental,
    entry: inlineRuntime ? { library } : { main: './entry.js', library },
    output: {
      path: context.getDist(incremental ? 'incremental' : 'clean'),
      filename: '[name].mjs',
      chunkFilename: '[id].mjs',
      module: true,
      uniqueName: 'mf-ordered-startup-rebuild',
    },
    optimization: { runtimeChunk: inlineRuntime ? false : 'single' },
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

function prepareInput(context) {
  const input = context.getDist('input');
  fs.mkdirSync(input, { recursive: true });
  fs.writeFileSync(path.join(input, 'entry.js'), initialConsume);
  fs.writeFileSync(path.join(input, 'library.js'), 'export const answer = 42;');
  fs.writeFileSync(path.join(input, 'lib.js'), 'export default 42;');
  fs.writeFileSync(
    path.join(input, 'exposed.js'),
    'export default await Promise.resolve("exposed");',
  );
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

export default [
  {
    name: 'container-startup',
    description: 'should update container startup and preserve library exports when the ordered consume moves between chunks',
    options(context) {
      prepareInput(context);
      return config(context, true);
    },
    compiler(_context, compiler) {
      compiler.outputFileSystem = fs;
    },
    async build(context, compiler) {
      const entry = path.join(context.getDist('input'), 'entry.js');
      await build(compiler);
      checkContainer(compiler.outputPath, 'async');
      checkLibrary(compiler.outputPath);

      fs.writeFileSync(entry, lazyConsume);
      await build(compiler, entry);
      const clean = rspack(config(context, false));
      try {
        await build(clean);
        checkContainer(clean.outputPath, 'sync');
        checkContainer(compiler.outputPath, 'sync');
        checkLibrary(clean.outputPath, 'require');
        checkLibrary(compiler.outputPath, 'require');
      } finally {
        await closeCompiler(clean);
      }

      fs.writeFileSync(entry, initialConsume);
      await build(compiler, entry);
      checkContainer(compiler.outputPath, 'async');
      checkLibrary(compiler.outputPath);
    },
  },
  {
    name: 'inline-library-startup',
    description: 'should export the ordered shared value from an ESM library with an inline runtime',
    options(context) {
      prepareInput(context);
      return config(context, false, true);
    },
    compiler(_context, compiler) {
      compiler.outputFileSystem = fs;
    },
    check({ compiler }) {
      checkLibrary(compiler.outputPath);
    },
  },
];
