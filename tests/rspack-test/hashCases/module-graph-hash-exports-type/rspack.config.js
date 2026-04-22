const path = require('node:path');

const hashes = new Set();

class CheckHashPlugin {
  apply(compiler) {
    compiler.hooks.compilation.tap(CheckHashPlugin.name, (compilation) => {
      compilation.hooks.processAssets.tap(CheckHashPlugin.name, () => {
        const sharedModule = Array.from(compilation.modules).find((module) =>
          module.resource?.endsWith('shared.cjs'),
        );
        if (!sharedModule) {
          throw new Error('no shared.cjs found');
        }
        const hash = compilation.chunkGraph.getModuleHash(
          sharedModule,
          'runtime',
        );
        if (!hash) {
          throw new Error('no hash for shared.cjs');
        }
        hashes.add(hash);
        expect(hashes.size).toBe(1);
      });
    });
  }
}

function config(name) {
  return {
    name,
    mode: 'production',
    devtool: false,
    entry: {
      loose: './loose.cjs',
      strict: './strict.mjs',
    },
    output: {
      path: path.join(__dirname, 'dist', name),
      filename: '[name].[contenthash].js',
    },
    optimization: {
      chunkIds: 'named',
      moduleIds: 'named',
      minimize: false,
      realContentHash: false,
      runtimeChunk: {
        name: 'runtime',
      },
    },
    plugins: [new CheckHashPlugin()],
  };
}

module.exports = Array.from({ length: 8 }).map((_, i) => config(String(i)));
