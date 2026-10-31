import assert from 'node:assert/strict';
import fs from 'node:fs';
import { createServer } from 'node:http';
import { createRequire } from 'node:module';
import os from 'node:os';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { rspack, lazyCompilationMiddleware } from '@rspack/core';

const nativeRequire = createRequire(import.meta.url);

export default [false, true].flatMap(esm => [false, true].flatMap(lazy => [false, true].map(importMeta => {
  let directory;
  const extension = esm ? 'mjs' : 'cjs';
  const hot = importMeta ? 'import.meta.webpackHot' : 'module.hot';
  return {
    description: `retains ${extension} entry through lazy=${lazy} activation and accepted edits with ${hot}`,
    options() {
      directory = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), 'rspack-lazy-hmr-')));
      fs.writeFileSync(path.join(directory, 'index.js'), `
let promise;
export const load = () => promise ??= import('./lazy.js');
${hot}.accept('./lazy.js', () => { promise = import('./lazy.js'); });
export const update = () => ${hot}.check(true);
export const hash = () => __webpack_hash__;
`);
      fs.writeFileSync(path.join(directory, 'lazy.js'), 'export const value = 42;');
      return {
        mode: 'development',
        context: directory,
        target: 'node',
        entry: './index.js',
        devtool: false,
        experiments: { outputModule: esm },
        output: {
          path: path.join(directory, 'dist'),
          filename: `index.${extension}`,
          chunkFilename: `[id].${extension}`,
          module: esm,
          library: { type: esm ? 'module' : 'commonjs2' },
          publicPath: 'auto',
          hotUpdateChunkFilename: `[id].[fullhash].hot-update.${extension}`,
        },
        plugins: [new rspack.HotModuleReplacementPlugin()],
        lazyCompilation: lazy ? { imports: true, entries: false } : false,
      };
    },
    async build(context, compiler) {
      compiler.outputFileSystem = fs;
      let middleware;
      const server = createServer((req, res) => middleware(req, res));
      await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
      if (lazy) {
        compiler.options.lazyCompilation.serverUrl = `http://127.0.0.1:${server.address().port}`;
        middleware = lazyCompilationMiddleware(compiler);
      }
      const results = [];
      let receive;
      let runtime;
      compiler.hooks.done.tapPromise('LazyHmrIdentity', async stats => {
        assert.equal(stats.hasErrors(), false, stats.toString());
        if (runtime) {
          while (runtime.hash() !== stats.hash) {
            assert.ok(await runtime.update(), 'Expected an HMR update');
          }
        }
      });
      const watching = compiler.watch({}, (error, stats) => {
        const result = { error, stats };
        if (receive) {
          const resolve = receive;
          receive = undefined;
          resolve(result);
        } else results.push(result);
      });
      const next = async () => {
        const { error, stats } = results.length ? results.shift() : await new Promise(resolve => { receive = resolve; });
        if (error) throw error;
        return stats;
      };
      try {
        await next();
        const output = path.join(directory, 'dist', `index.${extension}`);
        runtime = esm ? await import(pathToFileURL(output).href) : nativeRequire(output);
        const activation = runtime.load();
        if (lazy) await next();
        assert.equal((await activation).value, 42);
        fs.writeFileSync(path.join(directory, 'lazy.js'), 'export const value = 43;');
        await next();
        assert.equal((await runtime.load()).value, 43);
      } finally {
        await new Promise(resolve => watching.close(resolve));
        await new Promise(resolve => server.close(resolve));
        fs.rmSync(directory, { recursive: true, force: true });
      }
    },
  };
})));
