import { defineConfig, definePlugin } from '@rspack/cli';
import { rspack } from '@rspack/core';

const {
  Compilation,
  sources: { RawSource },
} = rspack;

export default defineConfig({
  target: ['web', 'es2020'],
  entry: {
    a: './a',
  },
  output: {
    filename: '[name].mjs',
    module: true,
  },
  experiments: {
    sourceImport: true,
  },
  externalsType: 'module-import',
  externals: [
    ({ request }, callback) => {
      if (request === './static.wasm') {
        return callback(undefined, './external/static.wasm');
      }
      if (request === './dynamic.wasm') {
        return callback(undefined, './external/dynamic.wasm');
      }
      callback();
    },
  ],
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.hooks.compilation.tap(
          'check-source-phase-externals',
          (compilation) => {
            compilation.hooks.processAssets.tap(
              {
                name: 'check-source-phase-externals',
                stage: Compilation.PROCESS_ASSETS_STAGE_ADDITIONAL,
              },
              () => {
                const content = compilation
                  .getAssets()
                  .filter((asset) => /\.(?:mjs|js)$/.test(asset.name))
                  .map((asset) => asset.source.source())
                  .join('\n');
                expect(content).toMatch(
                  /import source __rspack_external_.+ from "\.\/external\/static\.wasm";/,
                );
                expect(content).toContain(
                  'import.source("./external/dynamic.wasm").then',
                );
                expect(content).not.toContain(
                  'import("./external/static.wasm").then',
                );
                expect(content).not.toContain(
                  'import("./external/dynamic.wasm").then',
                );
                expect(content).not.toContain('import * as __rspack_external_');

                compilation.emitAsset(
                  'assertions.txt',
                  new RawSource('source phase externals preserved'),
                );
              },
            );
          },
        );
      },
    }),
  ],
});
