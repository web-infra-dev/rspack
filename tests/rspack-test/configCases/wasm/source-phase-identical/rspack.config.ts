import { defineConfig } from '@rspack/cli';

export default defineConfig({
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
    webassemblyModuleFilename: '[id].[hash].wasm',
  },
  experiments: {
    asyncWebAssembly: true,
    sourceImport: true,
  },
});
