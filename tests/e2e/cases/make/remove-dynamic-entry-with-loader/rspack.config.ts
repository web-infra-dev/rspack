import type { Configuration, Compiler } from '@rspack/core';
import rspack from '@rspack/core';

const sharedObj = {
  useFullEntry: true,
};

export default {
  entry: async (): Promise<Record<string, { import: string }>> => {
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
    function (compiler: Compiler) {
      Object.assign(compiler, { __sharedObj: sharedObj });
    },
  ],
  devServer: {
    hot: true,
  },
  lazyCompilation: false,
} satisfies Configuration;
