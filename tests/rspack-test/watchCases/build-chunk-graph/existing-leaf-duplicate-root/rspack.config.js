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
          async (compilation) => {
            const leafSource = fs.readFileSync(
              path.resolve(compiler.context, 'leaf.js'),
              'utf-8',
            );
            const addEntry = (request, options) =>
              new Promise((resolve, reject) => {
                compilation.addEntry(
                  compiler.context,
                  rspack.EntryPlugin.createDependency(request),
                  options,
                  (error) => (error ? reject(error) : resolve()),
                );
              });
            const globalEntry = leafSource.includes('use-as-global-entry')
              ? './leaf.js'
              : './index.js';

            await addEntry(globalEntry, {});
            await addEntry('./index.js', {
              name: 'main',
              filename: leafSource.includes('use-entry-filename')
                ? 'renamed.js'
                : 'main.js',
            });
          },
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
