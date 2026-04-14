const fs = require('fs');
const path = require('path');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'production',
  entry: './index.js',
  output: {
    module: true,
    library: {
      type: 'module',
    },
  },
  externals: {
    external: ['./external.mjs', 'inner'],
  },
  externalsType: 'module',
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.compilation.tap('Test', (compilation) => {
          compilation.hooks.processAssets.tap(
            {
              name: 'copy-plugin',
              stage:
                compiler.rspack.Compilation.PROCESS_ASSETS_STAGE_ADDITIONAL,
            },
            () => {
              const data = fs.readFileSync(
                path.resolve(__dirname, './external.mjs'),
              );
              compilation.emitAsset(
                'external.mjs',
                new compiler.rspack.sources.RawSource(data),
              );
            },
          );
        });
      },
    },
  ],
};
