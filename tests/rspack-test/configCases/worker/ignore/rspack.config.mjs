import fs from 'node:fs';
import path from 'node:path';

/** @type {import("../../../../").Configuration} */
export default {
  output: {
    assetModuleFilename: 'worker-[name].mjs',
    environment: {
      nodePrefixForCoreModules: false,
    },
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.compilation.tap('Test', (compilation) => {
          compilation.hooks.processAssets.tap(
            {
              name: 'copy-webpack-plugin',
              stage:
                compiler.rspack.Compilation.PROCESS_ASSETS_STAGE_ADDITIONAL,
            },
            () => {
              const data = fs.readFileSync(
                path.resolve(import.meta.dirname, './worker.js'),
              );

              compilation.emitAsset(
                'worker.mjs',
                new compiler.rspack.sources.RawSource(data),
              );
            },
          );
        });
      },
    },
  ],
};
