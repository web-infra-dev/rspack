const { ModuleFederationPlugin } = require("@rspack/core").container;
const path = require('path');

const common = {
  name: 'container_7',
  filename: 'container.js',
  exposes: {
    './ComponentA': './ComponentA',
    './App': './App',
    './noop': './emptyComponent',
  },
  shareStrategy: 'version-first',
  shared: {
    react: {
      singleton: true,
      requiredVersion: '0.1.2',
      strictVersion: true,
    },
    randomvalue: {
      request: 'react',
      import: 'react',
      shareKey: 'react',
      singleton: true,
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
        test: /ComponentA\.js$/,
        layer: 'react-layer',
      },
      {
        test: /react\.js$/,
        issuerLayer: 'react-layer',
        layer: 'react-layer',
        use: [
          {
            loader: path.resolve(__dirname, './layered-react-loader.js'),
          },
        ],
      },
    ],
  },
};

module.exports = [
  {
    ...commonConfig,
    output: {
      filename: '[name].js',
      uniqueName: '7-layers-full',
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
      uniqueName: '7-layers-full-mjs',
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
