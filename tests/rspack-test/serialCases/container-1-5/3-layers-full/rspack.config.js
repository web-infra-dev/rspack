const { ModuleFederationPlugin } = require("@rspack/core").container;

const common = {
  name: 'container_a',
  filename: 'container.js',
  exposes: {
    './ComponentA': './ComponentA',
  },
  shared: {
    react: {
      singleton: true,
      requiredVersion: false,
      version: false,
    },
  },
};

const commonConfig = {
  entry: './index.js',
  mode: 'development',
  devtool: false,
};

module.exports = [
  {
    ...commonConfig,
    output: {
      filename: '[name].js',
      uniqueName: '3-layers-full',
    },
    plugins: [
      new ModuleFederationPlugin({
        library: { type: 'commonjs-module' },
        ...common,
      }),
    ],
  },
  {
    ...commonConfig,
    experiments: {
      outputModule: true,
    },
    output: {
      module: true,
      filename: 'module/[name].mjs',
      uniqueName: '3-layers-full-mjs',
    },
    plugins: [
      new ModuleFederationPlugin({
        ...common,
        library: { type: 'module' },
        filename: 'module/container.mjs',
      }),
    ],
    target: 'node14',
  },
];
