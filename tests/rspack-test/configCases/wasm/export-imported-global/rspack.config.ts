import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: './index',
  module: {
    rules: [
      {
        test: /\.wat$/,
        loader: 'wast-loader',
        type: 'webassembly/async',
      },
    ],
  },
  experiments: {
    asyncWebAssembly: true,
  },
});
