const assert = require('assert');
const { ModuleFederationPlugin } = require('@rspack/core').container;

function filename(pathData) {
  return pathData.chunk
    ? 'real-chunk/[name].js'
    : 'missing-chunk/deep/[name].js';
}

class Plugin {
  apply(compiler) {
    compiler.hooks.done.tap(
      'federation-root-output-dir-filename-function',
      (stats) => {
        const assets = stats.compilation.getAssets();
        const runtimeAssets = assets.filter((asset) =>
          String(asset.source.source()).includes('rootOutputDir'),
        );

        assert(runtimeAssets.length > 0);
        for (const asset of runtimeAssets) {
          assert(
            asset.name.startsWith('real-chunk/'),
            `unexpected asset name: ${asset.name}`,
          );

          const source = String(asset.source.source());
          assert(
            source.includes('rootOutputDir: "../"'),
            source.match(/rootOutputDir: "[^"]*"/)?.[0] ?? source,
          );
          assert(!source.includes('rootOutputDir: "../../"'));
        }
      },
    );
  }
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  entry: './index.js',
  output: {
    filename,
    chunkFilename: filename,
    uniqueName: 'federation-root-output-dir-filename-function',
  },
  plugins: [
    new ModuleFederationPlugin({
      name: 'container',
      filename: 'real-chunk/container.js',
      exposes: {
        './module': './module.js',
      },
    }),
    new Plugin(),
  ],
};
