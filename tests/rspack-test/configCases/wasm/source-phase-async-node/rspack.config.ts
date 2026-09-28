import { defineConfig } from '@rspack/cli';

export default defineConfig([
  {
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
      module: true,
      webassemblyModuleFilename: '[id].[hash].wasm',
    },
    experiments: {
      asyncWebAssembly: true,
      sourceImport: true,
    },
  },
  {
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
      webassemblyModuleFilename: '[id].[hash].wasm',
    },
    experiments: {
      asyncWebAssembly: true,
      sourceImport: true,
    },
  },
]);
