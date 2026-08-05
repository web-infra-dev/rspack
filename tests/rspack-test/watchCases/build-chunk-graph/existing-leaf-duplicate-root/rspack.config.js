const rspack = require('@rspack/core');
const fs = require('fs');
const path = require('path');

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
              const leafSource = fs.readFileSync(
                path.resolve(compiler.context, 'leaf.js'),
                'utf-8',
              );
              const globalEntry = leafSource.includes('use-as-global-entry')
                ? './leaf.js'
                : './index.js';
              compilation.addEntry(
                compiler.context,
                rspack.EntryPlugin.createDependency(globalEntry),
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
