import { defineConfig } from '@rspack/cli';

export default defineConfig({
  name: 'web',
  target: ['web', 'node'],
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
    module: true,
    webassemblyModuleFilename: '[id].[hash].wasm',
  },
  experiments: {
    asyncWebAssembly: true,
    sourceImport: true,
  },
});
