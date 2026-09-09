const { CssExtractRspackPlugin } = require('@rspack/core');

/** @typedef {import('@rspack/core').Configuration} Configuration */

/**
 * @param {string} name
 * @param {Partial<Configuration>} config
 * @param {(string | undefined)[]} expectedCssChunks
 * @param {boolean} [requeue]
 * @returns {Configuration}
 */
function createConfig(name, config, expectedCssChunks, requeue = false) {
  return {
    mode: 'production',
    target: 'web',
    ...config,
    name,
    entry: config.entry || (requeue ? './requeue' : './index'),
    output: {
      filename: `${name}-[name].js`,
      chunkFilename: `${name}-[name].js`,
    },
    module: {
      rules: [
        {
          test: /\.css$/,
          type: 'javascript/auto',
          use: [CssExtractRspackPlugin.loader, 'css-loader', './repeat-loader'],
        },
        {
          test: /shared\.js$/,
          loader: '../min-remaining-size/size-loader',
          options: { size: 200 },
        },
      ],
    },
    optimization: {
      minimize: false,
      concatenateModules: false,
      inlineExports: false,
      sideEffects: false,
      splitChunks: {
        ...(requeue ? { minSize: 100, enforceSizeThreshold: 0 } : {}),
        ...config.optimization?.splitChunks,
        cacheGroups: {
          default: false,
          // Select local fixture files instead of depending on shaka-player in node_modules.
          defaultVendors: {
            test: /(?:controls\.css|shared\.js)$/,
            reuseExistingChunk: true,
          },
        },
      },
    },
    plugins: [
      new CssExtractRspackPlugin({
        filename: `${name}-[name].css`,
        chunkFilename: `${name}-[name].css`,
      }),
      function () {
        this.hooks.compilation.tap(name, (compilation) => {
          compilation.hooks.afterSeal.tap(name, () => {
            const chunks = Array.from(compilation.chunks);
            const cssChunks = chunks.filter((chunk) =>
              Array.from(chunk.files).some((file) => file.endsWith('.css')),
            );
            expect({
              [name]: cssChunks.map((chunk) => chunk.name).sort(),
            }).toEqual({
              [name]: expectedCssChunks,
            });
            for (const chunkName of expectedCssChunks) {
              if (chunkName) {
                expect(
                  compilation.getAsset(`${name}-${chunkName}.css`),
                ).toBeDefined();
              }
            }
            if (requeue) {
              const sharedChunks = chunks.filter((chunk) =>
                Array.from(
                  compilation.chunkGraph.getChunkModulesIterable(chunk),
                ).some((module) =>
                  module.nameForCondition()?.endsWith('shared.js'),
                ),
              );
              expect(sharedChunks.map((chunk) => chunk.name)).toEqual([
                'controls',
              ]);
            }
          });
        });
      },
    ],
  };
}

/** @type {Configuration[]} */
module.exports = [
  createConfig('production-default', {}, ['controls']),
  createConfig('development-default', { mode: 'development' }, [undefined]),
  createConfig(
    'explicit-zero',
    { optimization: { splitChunks: { minRemainingSize: 0 } } },
    [undefined],
  ),
  createConfig('retry-css-candidate', {}, [undefined], true),
  createConfig(
    'retry-after-reuse',
    {
      entry: './reused-requeue',
      optimization: {
        splitChunks: {
          minSize: 0,
          minRemainingSize: 90,
          enforceSizeThreshold: 0,
        },
      },
    },
    ['reuse', undefined],
  ),
];
