import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'node',
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
    webassemblyModuleFilename: '[id].[hash:base64:8].wasm',
  },
  experiments: {
    asyncWebAssembly: true,
  },
});
