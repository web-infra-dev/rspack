import assert from 'node:assert/strict';
import { runInNewContext } from 'node:vm';
import { defineConfig, definePlugin } from '@rspack/cli';
import { Compilation, experiments } from '@rspack/core';

export default [false, true].flatMap((usedExports, exportIndex) =>
  [false, true].map((workerFirst, orderIndex) => {
    const index = exportIndex * 2 + orderIndex;
    const normalImport =
      "export const load = () => import(/* webpackChunkName: 'shared' */ './shared');";
    const workerImport =
      "export const worker = () => new Worker(/* webpackChunkName: 'shared' */ new URL('./shared', import.meta.url));";
    return defineConfig({
      mode: 'production',
      target: 'web',
      devtool: false,
      cache: false,
      incremental: false,
      entry: './index.js',
      output: {
        filename: `main-${index}.js`,
        chunkFilename: `[name]-${index}.js`,
        publicPath: '/path/',
      },
      optimization: {
        splitChunks: false,
        minimize: false,
        concatenateModules: false,
        usedExports,
        moduleIds: 'named',
        chunkIds: 'named',
      },
      plugins: [
        new experiments.VirtualModulesPlugin({
          'index.js': `${workerFirst ? workerImport : normalImport}
            export const later = () => import(/* webpackChunkName: 'bridge' */ './bridge');`,
          'bridge.js': workerFirst ? normalImport : workerImport,
          'shared.js': `import { value } from './dep';
            globalThis.entryRuns = (globalThis.entryRuns || 0) + 1;
            globalThis.entryValue = value;
            export { value };`,
          'dep.js': 'export const value = 42;',
        }),
        definePlugin({
          apply(compiler) {
            compiler.hooks.compilation.tap(
              'AssertSharedWorkerEntry',
              (compilation) => {
                compilation.hooks.processAssets.tap(
                  {
                    name: 'AssertSharedWorkerEntry',
                    stage: Compilation.PROCESS_ASSETS_STAGE_REPORT,
                  },
                  () => {
                    const chunk = compilation.namedChunks.get('shared')!;
                    assert.equal([...chunk.groupsIterable].length, 2);
                    const entries = [
                      ...compilation.chunkGraph.getChunkEntryModulesIterable(
                        chunk,
                      ),
                    ];
                    assert.equal(entries.length, 1);
                    assert.match(entries[0].identifier(), /[\\/]shared\.js$/);

                    const source = compilation
                      .getAsset(`shared-${index}.js`)!
                      .source.source();
                    const worker: {
                      self?: unknown;
                      entryRuns?: number;
                      entryValue?: number;
                    } = {};
                    worker.self = worker;
                    runInNewContext(source.toString(), worker, {
                      timeout: 1000,
                    });
                    assert.equal(
                      worker.entryRuns,
                      1,
                      'execute the existing module as the worker entry',
                    );
                    assert.equal(worker.entryValue, 42);
                  },
                );
              },
            );
          },
        }),
      ],
    });
  }),
);
