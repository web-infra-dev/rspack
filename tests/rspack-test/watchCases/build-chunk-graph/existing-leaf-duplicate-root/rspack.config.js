const rspack = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'development',
  target: 'node',
  entry: {
    keeper: './leaf.js',
    main: './index.js',
  },
  output: {
    filename: '[name].js',
  },
  optimization: {
    minimize: false,
    splitChunks: false,
  },
  incremental: {
    buildChunkGraph: true,
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.make.tapPromise(
          'duplicate-entry-root',
          (compilation) =>
            new Promise((resolve, reject) => {
              compilation.addEntry(
                compiler.context,
                rspack.EntryPlugin.createDependency('./index.js'),
                {},
                (error) => (error ? reject(error) : resolve()),
              );
            }),
        );
      },
    },
  ],
  stats: {
    preset: 'verbose',
    logging: 'verbose',
    loggingDebug: [/codeSplittingCache/, /incremental/],
  },
};
