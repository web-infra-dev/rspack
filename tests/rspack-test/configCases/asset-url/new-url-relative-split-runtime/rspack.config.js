const { Compilation } = require('@rspack/core');

/** @type {import('@rspack/core').Configuration} */
module.exports = {
  mode: 'development',
  devtool: false,
  entry: {
    a: './a.js',
    b: './b.js',
  },
  output: {
    filename: '[name].js',
    assetModuleFilename: 'assets/[name][ext]',
  },
  optimization: {
    concatenateModules: false,
    splitChunks: {
      cacheGroups: {
        assetOnly: {
          test: /asset\.txt$/,
          name: 'asset-only',
          chunks: 'all',
          enforce: true,
        },
      },
    },
  },
  module: {
    parser: {
      javascript: {
        url: 'new-url-relative',
      },
    },
  },
  plugins: [
    (compiler) => {
      compiler.hooks.thisCompilation.tap(
        'NewUrlRelativeSplitRuntimeTest',
        (compilation) => {
          compilation.hooks.processAssets.tap(
            {
              name: 'NewUrlRelativeSplitRuntimeTest',
              stage: Compilation.PROCESS_ASSETS_STAGE_REPORT,
            },
            (assets) => {
              expect(Object.keys(assets).sort()).toEqual([
                'a.js',
                'assets/asset.txt',
                'b.js',
              ]);

              for (const filename of ['a.js', 'b.js']) {
                const source = assets[filename].source().toString();
                expect(source).toContain('assets/asset.txt');
                expect(source).not.toContain(
                  'RSPACK_AUTO_URL_STATIC_PLACEHOLDER_',
                );
              }

              const assetChunk = compilation.namedChunks.get('asset-only');
              expect(assetChunk).toBeDefined();
              const modules = [
                ...compilation.chunkGraph.getChunkModulesIterable(assetChunk),
              ];
              expect(modules).toHaveLength(1);
              expect(modules[0].type).toBe('asset/resource');
              expect([...assetChunk.files]).toEqual([]);
              expect([...assetChunk.auxiliaryFiles]).toEqual([
                'assets/asset.txt',
              ]);
            },
          );
        },
      );
    },
  ],
};
