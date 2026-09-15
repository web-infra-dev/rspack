/** @typedef {import('@rspack/core').Configuration} Configuration */

/**
 * @param {string} name
 * @param {Partial<Configuration>} config
 * @param {string[]} expectedChunks
 * @param {import('@rspack/core').OptimizationSplitChunksCacheGroup} [group]
 * @returns {Configuration}
 */
function createConfig(name, config, expectedChunks, group = {}) {
  return {
    mode: 'production',
    target: 'web',
    ...config,
    name,
    output: {
      filename: `${name}-[name].js`,
      chunkFilename: `${name}-[name].js`,
    },
    module: {
      rules: [
        {
          test: /async\.js$/,
          loader: './size-loader',
          options: { size: 80 },
        },
        {
          test: /shared\.js$/,
          loader: './size-loader',
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
        minSize: 100,
        enforceSizeThreshold: 0,
        ...config.optimization?.splitChunks,
        cacheGroups: {
          default: false,
          defaultVendors: false,
          ...config.optimization?.splitChunks?.cacheGroups,
          shared: {
            test: /shared\.js$/,
            name: 'shared',
            reuseExistingChunk: true,
            ...group,
          },
        },
      },
    },
    plugins: [
      function () {
        this.hooks.compilation.tap(name, (compilation) => {
          compilation.hooks.afterSeal.tap(name, () => {
            const chunks = Array.from(compilation.chunks)
              .filter((chunk) =>
                Array.from(
                  compilation.chunkGraph.getChunkModulesIterable(chunk),
                ).some((module) =>
                  module.nameForCondition()?.endsWith('shared.js'),
                ),
              )
              .map((chunk) => chunk.name)
              .sort();
            expect({ [name]: chunks }).toEqual({ [name]: expectedChunks });
          });
        });
      },
    ],
  };
}

/** @type {Configuration[]} */
module.exports = [
  createConfig('production-default', {}, ['async']),
  createConfig('none-default', { mode: 'none' }, ['async']),
  createConfig('development-default', { mode: 'development' }, ['shared']),
  createConfig(
    'global-zero',
    { optimization: { splitChunks: { minRemainingSize: 0 } } },
    ['shared'],
  ),
  createConfig(
    'global-object',
    {
      optimization: { splitChunks: { minRemainingSize: { javascript: 100 } } },
    },
    ['async'],
  ),
  createConfig('group-zero', {}, ['shared'], { minRemainingSize: 0 }),
  createConfig('group-object', { mode: 'development' }, ['async'], {
    minRemainingSize: { javascript: 100 },
  }),
  createConfig('group-min-size-fallback', { mode: 'development' }, ['async'], {
    minSize: 100,
  }),
  createConfig(
    'group-min-size-over-global-zero',
    { optimization: { splitChunks: { minRemainingSize: 0 } } },
    ['async'],
    { minSize: 100 },
  ),
  createConfig(
    'remaining-equality',
    { optimization: { splitChunks: { minRemainingSize: 80 } } },
    ['shared'],
  ),
  createConfig('enforced-group', {}, ['shared'], { enforce: true }),
  createConfig('enforced-group-explicit-remaining', {}, ['async'], {
    enforce: true,
    minRemainingSize: 100,
  }),
  createConfig('enforced-group-min-size-fallback', {}, ['async'], {
    enforce: true,
    minSize: 100,
  }),
  createConfig(
    'enforced-group-no-global-threshold',
    { optimization: { splitChunks: { enforceSizeThreshold: 1 } } },
    ['async'],
    { enforce: true, minRemainingSize: 100 },
  ),
  createConfig(
    'threshold-equality',
    { optimization: { splitChunks: { enforceSizeThreshold: 200 } } },
    ['shared'],
  ),
  createConfig(
    'missing-source-type',
    {
      optimization: {
        splitChunks: { minSize: 0, minRemainingSize: { css: 100 } },
      },
    },
    ['shared'],
  ),
  createConfig('multiple-source-chunks', { entry: './multiple' }, ['shared']),
  createConfig(
    'remaining-after-request-limits',
    {
      entry: './max-requests',
      optimization: {
        splitChunks: {
          cacheGroups: {
            prelude: {
              test: /prelude\.js$/,
              name: 'prelude',
              priority: 100,
              enforce: true,
            },
          },
        },
      },
    },
    ['one', 'two'],
    { maxAsyncRequests: 2 },
  ),
  createConfig(
    'reused-destination',
    { entry: './reuse' },
    ['async', 'shared'],
    {
      name: false,
    },
  ),
  createConfig('reused-destination-allowed', { entry: './reuse' }, ['shared'], {
    name: false,
    minRemainingSize: 0,
  }),
];
