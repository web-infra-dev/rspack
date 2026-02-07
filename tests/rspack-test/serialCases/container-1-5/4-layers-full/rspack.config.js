const { ModuleFederationPlugin } = require("@rspack/core").container;

const common = {
  name: 'container_b',
  filename: 'container.js',
  shared: {
    react: {
      singleton: true,
      requiredVersion: false,
      version: false,
      import: false,
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
      uniqueName: '4-layers-full',
    },
    plugins: [
      new ModuleFederationPlugin({
        ...common,
        library: { type: 'commonjs-module' },
        remotes: {
          containerA: '../3-layers-full/container.js',
        },
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
      uniqueName: '4-layers-full-mjs',
    },
    plugins: [
      new ModuleFederationPlugin({
        ...common,
        library: { type: 'module' },
        filename: 'module/container.mjs',
        remotes: {
          containerA: '../../3-layers-full/module/container.mjs',
        },
      }),
    ],
    target: 'node14',
  },
];
