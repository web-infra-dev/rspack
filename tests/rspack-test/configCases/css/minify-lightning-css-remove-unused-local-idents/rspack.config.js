const { rspack } = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
const common = {
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  target: 'web',
  node: {
    __dirname: false,
    __filename: false,
  },
  module: {
    generator: {
      'css/auto': {
        localIdentName: '[path][name]-[local]',
        exportsOnly: false,
        exportsConvention: 'camel-case',
      },
    },
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
  optimization: {
    minimize: true,
    minimizer: [new rspack.LightningCssMinimizerRspackPlugin()],
  },
};

module.exports = [
  {
    ...common,
    plugins: [
      new rspack.DefinePlugin({
        EXPORTS_ONLY: false,
      }),
    ],
  },
  {
    ...common,
    plugins: [
      new rspack.DefinePlugin({
        EXPORTS_ONLY: true,
      }),
    ],
    module: {
      generator: {
        'css/auto': {
          localIdentName: '[path][name]-[local]',
          exportsOnly: true,
          exportsConvention: 'camel-case',
        },
      },
      rules: [
        {
          test: /\.css$/,
          type: 'css/auto',
        },
      ],
    },
    optimization: {
      minimize: true,
      concatenateModules: true,
      minimizer: [new rspack.LightningCssMinimizerRspackPlugin()],
    },
  },
];
