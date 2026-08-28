const { isMainThread } = require('node:worker_threads');
const { NormalModule } = require('@rspack/core');
const mainOptions = require('./main-options');

module.exports = {
  loader: {
    customValue: 'context',
    customTransform(value) {
      return `custom:${value}`;
    },
  },
  module: {
    rules: [
      {
        test: /value\.js$/,
        use: [
          {
            loader: './loader.js',
            parallel: { maxWorkers: 2 },
            options: {
              prefix: 'prefix:',
              map: new Map([['key', 'structured-clone']]),
              typed: new Uint8Array([1, 2, 3]),
              url: new URL('https://rspack.dev/loader-options'),
              transform(value) {
                return this.prefix + value;
              },
              invoke(value, callback) {
                return callback(value);
              },
              fail() {
                throw new TypeError('function option failed');
              },
            },
          },
        ],
      },
      {
        test: /main\.js$/,
        use: [
          {
            loader: './main-loader.js',
            options: mainOptions,
          },
        ],
      },
    ],
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.compilation.tap(
          'parallel-loader-hook',
          (compilation) => {
            NormalModule.getCompilationHooks(compilation).loader.tap(
              'parallel-loader-hook',
              (loaderContext) => {
                loaderContext.hookValue = 'context';
                loaderContext.hookMainThread = isMainThread;
                loaderContext.hookTransform = (value) => `hook:${value}`;
              },
            );
          },
        );
      },
    },
  ],
};
