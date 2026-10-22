import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'web',
  module: {
    rules: [
      {
        test: /\.wat$/,
        loader: 'wast-loader',
        type: 'webassembly/async',
      },
    ],
  },
  output: {
    chunkFilename: 'chunks/[name].async.js',
    webassemblyModuleFilename: '[id].[hash].async.wasm',
  },
  experiments: {
    asyncWebAssembly: true,
  },
});
