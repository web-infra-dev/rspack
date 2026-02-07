const { ModuleFederationPlugin } = require("@rspack/core").container;

const common = {
  name: 'container_6',
  filename: 'container.js',
  remotes: {
    containerA: {
      external: '../5-layers-full/container.js',
      shareScope: ['react-layer', 'default'],
    },
  },
  shared: {
    react: {
      request: 'react',
      import: false,
      shareKey: 'react',
      singleton: true,
      requiredVersion: false,
      layer: 'react-layer',
      issuerLayer: 'react-layer',
      shareScope: 'react-layer',
    },
  },
};

const commonConfig = {
  entry: './index.js',
  mode: 'development',
  devtool: false,
  experiments: {
    layers: true,
  },
  module: {
    rules: [
      {
        test: /\.js$/,
        layer: 'react-layer',
      },
    ],
  },
};

module.exports = [
  {
    ...commonConfig,
    output: {
      filename: '[name].js',
      uniqueName: '6-layers-full',
    },
    plugins: [
      new ModuleFederationPlugin({
        ...common,
        library: { type: 'commonjs-module' },
      }),
    ],
  },
  {
    ...commonConfig,
    experiments: {
      ...commonConfig.experiments,
      outputModule: true,
    },
    output: {
      module: true,
      filename: 'module/[name].mjs',
      uniqueName: '6-layers-full-mjs',
    },
    plugins: [
      new ModuleFederationPlugin({
        ...common,
        library: { type: 'module' },
        filename: 'module/container.mjs',
        remotes: {
          containerA: {
            external: '../../5-layers-full/module/container.mjs',
          },
        },
      }),
    ],
    target: 'node14',
  },
];
