import { defineConfig, definePlugin } from '@rspack/cli';
import { rspack, type RsdoctorPluginData } from '@rspack/core';

const {
  experiments: { RsdoctorPlugin },
} = rspack;

export default defineConfig({
  entry: {
    a: './a.js',
    b: './b.js',
  },
  output: {
    filename: '[name].js',
  },
  optimization: {
    chunkIds: 'named',
    moduleIds: 'named',
  },
  plugins: [
    new RsdoctorPlugin({
      moduleGraphFeatures: false,
      chunkGraphFeatures: ['graph', 'assets'],
    }),
    definePlugin({
      apply(compiler) {
        compiler.hooks.compilation.tap('TestPlugin::Assets', (compilation) => {
          const hooks = RsdoctorPlugin.getCompilationHooks(compilation);
          hooks.assets.tap('TestPlugin::Assets', (data) => {
            const { assets } = data;
            expect(assets.length).toBe(4);
            const assetsInfo = assets.map((a) => ({
              size: a.size,
              path: a.path,
            }));
            assetsInfo.sort((a, b) => (a.path > b.path ? 1 : -1));
            if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
              expect(assetsInfo.map((i) => ({ path: i.path }))).toEqual([
                {
                  path: 'a.js',
                },
                {
                  path: 'b.js',
                },
                {
                  path: 'c_js.js',
                },
                {
                  path: 'd_js.js',
                },
              ]);
            } else {
              expect(assetsInfo.map((i) => ({ path: i.path }))).toEqual([
                {
                  path: 'a.js',
                },
                {
                  path: 'b.js',
                },
                {
                  path: 'c_js.js',
                },
                {
                  path: 'd_js.js',
                },
              ]);
            }
          });
        });
      },
    }),
    definePlugin({
      apply(compiler) {
        compiler.hooks.compilation.tap(
          'TestPlugin::ChunkAssets',
          (compilation) => {
            const hooks = RsdoctorPlugin.getCompilationHooks(compilation);
            let chunks: RsdoctorPluginData.RsdoctorChunk[] = [];
            hooks.chunkGraph.tap('TestPlugin::ChunkAssets', (data) => {
              chunks = data.chunks;
            });
            hooks.assets.tap('TestPlugin::Assets', (data) => {
              const { chunkAssets } = data;
              for (const chunk of chunks) {
                expect(
                  chunkAssets.find((a) => a.chunk === chunk.ukey)?.assets
                    .length,
                ).toBe(1);
              }
            });
          },
        );
      },
    }),
    definePlugin({
      apply(compiler) {
        compiler.hooks.compilation.tap(
          'TestPlugin::EntrypointAssets',
          (compilation) => {
            const hooks = RsdoctorPlugin.getCompilationHooks(compilation);
            let entrypoints: RsdoctorPluginData.RsdoctorEntrypoint[] = [];
            hooks.chunkGraph.tap('TestPlugin::EntrypointAssets', (data) => {
              entrypoints = data.entrypoints;
            });
            hooks.assets.tap('TestPlugin::Assets', (data) => {
              const { entrypointAssets } = data;
              for (const ep of entrypoints) {
                expect(
                  entrypointAssets.find((a) => a.entrypoint === ep.ukey)?.assets
                    .length,
                ).toBe(1);
              }
            });
          },
        );
      },
    }),
  ],
});
