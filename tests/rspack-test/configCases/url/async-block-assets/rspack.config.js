const { Compilation } = require('@rspack/core');

module.exports = [false, true].flatMap((split) =>
  [true, false].map((asyncChunks) => ({
    mode: 'development',
    devtool: false,
    target: 'web',
    entry: { main: './index.js', other: './other.js' },
    output: {
      filename: `[name]-${split}-${asyncChunks}.js`,
      chunkFilename: `async-${split}-${asyncChunks}-[id].js`,
      assetModuleFilename: '[name][ext]',
      publicPath: '/path/',
      asyncChunks,
    },
    module: {
      rules: [
        { test: /\.txt$/, dependency: 'url', type: 'asset/resource' },
        { test: /resource\.txt$/, type: 'asset/resource' },
        { test: /inline\.txt$/, type: 'asset/inline' },
        { test: /[/\\]source\.txt$/, type: 'asset/source' },
        {
          test: /custom\.txt$/,
          type: 'asset/resource',
          generator: { publicPath: '/custom/', filename: '[name][ext]' },
        },
        { test: /dynamic\.txt$/, type: 'asset/resource' },
      ],
    },
    optimization: {
      concatenateModules: false,
      splitChunks: split
        ? {
            cacheGroups: {
              assets: {
                test: /resource\.txt$/,
                name: 'shared-assets',
                chunks: 'all',
                enforce: true,
              },
            },
          }
        : false,
    },
    plugins: [
      (compiler) => {
        compiler.hooks.compilation.tap('CheckUrlAssetChunks', (compilation) => {
          compilation.hooks.processAssets.tap(
            {
              name: 'CheckUrlAssetChunks',
              stage: Compilation.PROCESS_ASSETS_STAGE_REPORT,
            },
            () => {
              const asset = [...compilation.modules].find(
                (module) => module.rawRequest === './resource.txt',
              );
              expect(
                compilation.chunkGraph
                  .getModuleChunks(asset)
                  .map((chunk) => chunk.name)
                  .sort(),
              ).toEqual(split ? ['shared-assets'] : ['main', 'other']);
              for (const name of split
                ? ['shared-assets']
                : ['main', 'other']) {
                const modules = compilation.chunkGraph.getChunkModulesIterable(
                  compilation.namedChunks.get(name),
                );
                expect(modules).toContain(asset);
              }
              // Asset URL dependencies stay synchronous after the factory probe.
              const issuer = [...compilation.modules].find(
                (module) => module.rawRequest === './shared.js',
              );
              expect(issuer.blocks).toHaveLength(0);
              expect(
                issuer.dependencies.some((dep) => dep.type === 'new URL()'),
              ).toBe(true);
            },
          );
          compilation.hooks.processAssets.tap(
            {
              name: 'CheckUrlAssetChunks',
              stage: Compilation.PROCESS_ASSETS_STAGE_REPORT,
            },
            () => {
              expect(
                compilation
                  .getAssets()
                  .filter((asset) => asset.name === 'resource.txt'),
              ).toHaveLength(1);
              expect(compilation.getAsset('inline.txt')).toBeUndefined();
              expect(compilation.getAsset('source.txt')).toBeUndefined();
              for (const name of ['main', 'other']) {
                const files = compilation.entrypoints
                  .get(name)
                  .chunks.flatMap((chunk) => [...chunk.auxiliaryFiles]);
                expect(files).toContain('resource.txt');
              }
              if (split) {
                const chunk = compilation.namedChunks.get('shared-assets');
                expect(
                  compilation.chunkGraph.getChunkModulesIterable(chunk),
                ).toHaveLength(1);
                expect([...chunk.auxiliaryFiles]).toContain('resource.txt');
              }
            },
          );
        });
      },
    ],
  })),
);
