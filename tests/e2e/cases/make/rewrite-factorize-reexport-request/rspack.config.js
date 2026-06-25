const { rspack } = require('@rspack/core');
const path = require('path');

const sharedObj = {
  time: 1,
};

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  entry: './index.js',
  context: __dirname,
  cache: true,
  experiments: {
    cache: true,
  },
  incremental: true,
  module: {
    rules: [
      {
        include: path.resolve(__dirname, 'reexport.js'),
        sideEffects: true,
      },
    ],
  },
  optimization: {
    mangleExports: false,
  },
  plugins: [
    new rspack.HtmlRspackPlugin(),
    {
      apply(compiler) {
        compiler.__sharedObj = sharedObj;
        compiler.hooks.compilation.tap(
          'PLUGIN',
          (_, { normalModuleFactory }) => {
            normalModuleFactory.hooks.resolve.tapPromise(
              'PLUGIN',
              async (resolveData) => {
                if (resolveData.request == './file.js') {
                  resolveData.request = `./loader.cjs?time=${sharedObj.time}!./file.js`;
                }
              },
            );
          },
        );
      },
    },
  ],
};
