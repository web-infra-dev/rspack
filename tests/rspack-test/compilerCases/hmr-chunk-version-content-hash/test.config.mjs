import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';
import { rspack } from '@rspack/core';

// An export change in an initial module changes its importer's module hash,
// without changing the async chunk's module code generation results.
export default ['web', 'node', 'module'].flatMap((target) =>
  ['webpack', 'rspack'].flatMap((runtimeMode) =>
    [false, true].flatMap((hashed) =>
      ['hmr', 'development', 'production'].map((mode) => ({
        name: `${target}-${runtimeMode}-${hashed ? 'contenthash' : 'stable'}-${mode}`,
        description:
          'keeps cached async chunks consistent with the runtime after an export is added',
        options(context) {
          const esm = target === 'module';
          const extension = esm ? 'mjs' : 'js';
          return {
            context: context.getDist('input'),
            entry: './index.js',
            mode: mode === 'production' ? 'production' : 'development',
            devtool: false,
            target: esm ? 'node' : target,
            experiments: { runtimeMode },
            output: {
              path: context.getDist('output'),
              module: esm,
              chunkLoading: esm ? 'import' : undefined,
              filename: `main.${extension}`,
              chunkFilename: hashed
                ? `[name].[contenthash].${extension}`
                : `[name].${extension}`,
              publicPath: '/',
              globalObject: 'globalThis',
              chunkLoadTimeout: 1000,
            },
            optimization: {
              moduleIds: 'named',
              chunkIds: 'named',
              minimize: false,
              concatenateModules: false,
              splitChunks: false,
            },
            plugins:
              mode === 'hmr' ? [new rspack.HotModuleReplacementPlugin()] : [],
          };
        },
        async build(context, compiler) {
          const input = context.getDist('input');
          fs.mkdirSync(input, { recursive: true });
          const write = (name, source) =>
            fs.writeFileSync(path.join(input, name), source);
          const format = 'export const label = (s) => s.toUpperCase();';
          write('format.js', format);
          write(
            'page.js',
            `import { label } from './format.js';
             export default () => label('chunk');`,
          );
          write(
            'index.js',
            `import { label } from './format.js';
             globalThis.app = {
               load: () => import(/* webpackChunkName: "page" */ './page.js')
                 .then((m) => label('app') + ' ' + m.default())
               ${mode === 'hmr' ? ', hot: module.hot' : ''}
             };
             ${mode === 'hmr' ? "module.hot.accept('./format.js');" : ''}`,
          );
          let assets;
          const build = async (source) => {
            write('format.js', source);
            const stats = await new Promise((resolve, reject) =>
              compiler.run(
                (error, stats) => (error ? reject(error) : resolve(stats)),
                { modifiedFiles: new Set([path.join(input, 'format.js')]) },
              ),
            );
            assert.equal(stats.hasErrors(), false, stats.toString());
            assets = Object.fromEntries(
              stats.compilation
                .getAssets()
                .map(({ name, source }) => [name, source.source().toString()]),
            );
            const chunk = [...stats.compilation.chunks].find(
              (chunk) => chunk.name === 'page',
            );
            const filename = [...chunk.files].find((file) =>
              /\.m?js$/.test(file),
            );
            return {
              filename,
              source: assets[filename],
              hash: chunk.contentHash.javascript,
            };
          };
          const start = async () => {
            const sandbox = vm.createContext({
              console,
              setTimeout,
              clearTimeout,
            });
            sandbox.fetch = async (url) => {
              const source = assets[path.basename(url)];
              return {
                ok: source !== undefined,
                status: source === undefined ? 404 : 200,
                json: async () => JSON.parse(source),
              };
            };
            const modules = new Map();
            const loadModule = async (url) => {
              const file = path.basename(url);
              if (!modules.has(file)) {
                const mod = new vm.SourceTextModule(assets[file], {
                  context: sandbox,
                  identifier: file,
                  importModuleDynamically: loadModule,
                });
                modules.set(file, mod);
                await mod.link(loadModule);
                await mod.evaluate();
              }
              return modules.get(file);
            };
            const require = (url) => {
              const file = path.basename(url);
              if (file.endsWith('.json')) return JSON.parse(assets[file]);
              const module = { exports: {} };
              vm.runInContext(
                `(function(require, module, exports) { ${assets[file]}\n})`,
                sandbox,
              )(require, module, module.exports);
              return module.exports;
            };
            require.cache = {};
            require.resolve = (url) => url;
            const scripts = [];
            const head = {
              appendChild(script) {
                scripts.push(script);
                script.parentNode = head;
                queueMicrotask(() => {
                  vm.runInContext(assets[path.basename(script.src)], sandbox);
                  script.onload?.({ type: 'load', target: script });
                });
              },
              removeChild(script) {
                scripts.splice(scripts.indexOf(script), 1);
              },
            };
            sandbox.document = {
              head,
              getElementsByTagName: () => scripts,
              createElement: () => ({
                setAttribute(key, value) {
                  this[key] = value;
                },
                getAttribute(key) {
                  return this[key];
                },
              }),
            };
            if (target === 'module') await loadModule('main.mjs');
            else if (target === 'node') require('main.js');
            else vm.runInContext(assets['main.js'], sandbox);
            return sandbox.app;
          };

          const initial = await build(format);
          assert.equal(await (await start()).load(), 'APP CHUNK');
          // This running application has not loaded the async chunk yet.
          const running = mode === 'hmr' ? await start() : undefined;
          const changed = await build(`${format}\nexport const added = 1;`);
          if (running) {
            assert.equal(
              changed.hash,
              initial.hash,
              'the version must follow the rendered chunk, not module hashes',
            );
            assert.equal(changed.source, initial.source);
            assert.equal(changed.filename, initial.filename);
          }
          assert.equal(
            await (await start()).load(),
            'APP CHUNK',
            'a fresh page must load',
          );
          if (running) {
            await running.hot.check(true);
            assert.equal(
              await running.load(),
              'APP CHUNK',
              'HMR must publish a matching version',
            );
          } else {
            for (const source of Object.values(assets)) {
              assert(
                !source.includes('__rspack_esm_hash'),
                'non-HMR chunks need no version export',
              );
              assert(
                !source.includes('__rspack_cjs_hash'),
                'non-HMR chunks need no version export',
              );
              assert(
                !source.includes('Chunk is from another build'),
                'non-HMR builds need no version check',
              );
            }
            if (target === 'web') {
              const sandbox = { globalThis: {} };
              vm.runInNewContext(changed.source, sandbox);
              const [chunks] = Object.values(sandbox.globalThis);
              assert.equal(
                chunks[0].length,
                2,
                'non-HMR JSONP payloads need no version tag',
              );
            }
          }
          const unchanged = await build(
            `${format}\nexport const added = 1;\n// unrelated comment`,
          );
          assert.equal(unchanged.filename, changed.filename);
          assert.equal(unchanged.hash, changed.hash);
          assert.equal(
            unchanged.source,
            changed.source,
            'unchanged chunks must remain reusable',
          );
        },
      })),
    ),
  ),
);
