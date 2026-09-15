import { rspack } from '@rspack/core';

const sharedObj = {
  time: 1,
};

/** @type {import("@rspack/core").Configuration} */
export default {
  entry: './index.js',
  context: import.meta.dirname,
  cache: true,
  experiments: {
    cache: true,
  },
  incremental: true,
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
