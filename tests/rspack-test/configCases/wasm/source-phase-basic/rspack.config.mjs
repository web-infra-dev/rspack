/** @type {import("@rspack/core").Configuration[]} */
export default [
  {
    target: 'async-node',
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
      chunkFilename: 'chunks/[name].async.mjs',
      webassemblyModuleFilename: '[id].[hash].module.async.wasm',
    },
    experiments: {
      asyncWebAssembly: true,
      sourceImport: true,
    },
  },
];
