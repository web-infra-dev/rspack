import type { Configuration, Compiler } from '@rspack/core';
import { rspack } from '@rspack/core';

export default {
  entry: {
    main: './src/index.js',
  },
  mode: 'development',
  plugins: [
    new rspack.HtmlRspackPlugin({
      template: './src/index.html',
    }),
    function test(compiler: Compiler) {
      const assets: string[] = [];
      Object.assign(compiler, { assets });
      compiler.hooks.assetEmitted.tap('test', (name) => {
        if (name.includes('.hot-update.')) {
          return;
        }
        assets.push(name);
      });
    },
  ],
  optimization: {
    chunkIds: 'named',
  },
  output: { clean: true },
  lazyCompilation: false,
} satisfies Configuration;
