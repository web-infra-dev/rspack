const { rspack } = require('@rspack/core');

module.exports = [false, true].flatMap((cache) =>
  [false, true].flatMap((parallel) =>
    [false, true].map((mixed) => ({
      mode: 'development',
      incremental: false,
      cache: cache ? { type: 'memory' } : false,
      experiments: {
        newCache: {
          codeGeneration: false,
          loader: cache,
          minimize: false,
        },
      },
      module: {
        rules: [
          {
            test: /input\.txt$/,
            type: 'javascript/auto',
            use: [
              {
                loader: require.resolve('./consumer-loader'),
                options: { name: `${cache}-${parallel}-${mixed}` },
                cache,
                parallel: parallel ? { maxWorkers: 1 } : false,
              },
              ...(mixed
                ? [{ loader: 'builtin:test-passthrough-loader', cache }]
                : []),
              { loader: require.resolve('./producer-loader') },
            ],
          },
        ],
      },
      plugins: [new rspack.DefinePlugin({ LOADER_CACHE_ENABLED: cache })],
    })),
  ),
);
