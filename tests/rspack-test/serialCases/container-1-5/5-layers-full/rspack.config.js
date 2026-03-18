const { ModuleFederationPlugin } = require("@rspack/core").container;

const common = {
  name: 'container_5',
  filename: 'container.js',
  exposes: {
    './ComponentA': './ComponentA',
  },
  shareScope: ['react-layer', 'default'],
  shared: {
    react: {
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
      {
        test: /react\.js$/,
        issuerLayer: 'react-layer',
        loader: require.resolve('./layered-react-loader'),
      },
    ],
  },
};

module.exports = [
  {
    ...commonConfig,
    output: {
      filename: '[name].js',
      uniqueName: '5-layers-full',
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
      uniqueName: '5-layers-full-mjs',
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
