const { rspack } = require('@rspack/core');

const sharedObj = {
  time: 1,
};

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  entry: './index.js',
  context: __dirname,
  cache: true,
  output: {
    module: true,
    library: {
      type: 'module',
    },
  },
  experiments: {
    cache: true,
  },
  incremental: true,
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
