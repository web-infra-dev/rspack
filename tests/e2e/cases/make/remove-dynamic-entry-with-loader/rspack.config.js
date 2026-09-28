import rspack from '@rspack/core';

const sharedObj = {
  useFullEntry: true,
};

/** @type {import("@rspack/core").Configuration} */
export default {
  entry: async () => {
    if (sharedObj.useFullEntry) {
      return {
        main: {
          import: './loader.js!./src/index1.js',
        },
        main2: {
          import: './loader.js!./src/index2.js',
        },
      };
    } else {
      return {
        main: {
          import: './loader.js!./src/index1.js',
        },
      };
    }
  },
  context: import.meta.dirname,
  mode: 'development',
  optimization: {
    runtimeChunk: 'single',
  },
  plugins: [
    new rspack.HtmlRspackPlugin(),
    function (compiler) {
      compiler.__sharedObj = sharedObj;
    },
  ],
  devServer: {
    hot: true,
  },
  lazyCompilation: false,
};
