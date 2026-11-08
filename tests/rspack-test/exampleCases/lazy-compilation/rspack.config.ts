import { defineConfig } from '@rspack/cli';
import { HotModuleReplacementPlugin } from '@rspack/core';

export default defineConfig({
  mode: 'development',
  entry: {
    main: './example.js',
  },
  cache: true,
  lazyCompilation: true,
  devServer: {
    hot: true,
    devMiddleware: {
      publicPath: '/dist/',
    },
  },
  module: {
    parser: {
      javascript: {
        exportsPresence: 'auto',
      },
    },
  },
  plugins: [new HotModuleReplacementPlugin()],
});
