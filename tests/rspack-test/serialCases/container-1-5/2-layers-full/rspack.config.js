const { ModuleFederationPlugin } = require("@rspack/core").container;
const path = require('path');

const common = {
  entry: {
    main: './index.js',
  },
  // optimization: {
  //   runtimeChunk: 'single',
  // },
};

const commonMF = {
  runtime: false,
  exposes: {
    './ComponentB': './ComponentB',
    './ComponentC': './ComponentC',
    './ComponentALayers': './ComponentALayers',
  },
  shared: {
    react: {
      version: '17.0.0',
      requiredVersion: '^17.0.0',
      singleton: true,
    },
    'layered-react': {
      request: 'react',
      import: 'react',
      shareKey: 'react',
      version: '17.0.0',
      requiredVersion: '^17.0.0',
      singleton: true,
      layer: 'layered-components',
      issuerLayer: 'layered-components',
      shareScope: 'layered-components',
    },
  },
};

const commonConfig = {
  ...common,
  devtool: false,
  experiments: {
    layers: true,
  },
  module: {
    rules: [
      {
        test: /ComponentALayers\.js$/,
        layer: 'layered-components',
      },
      {
        test: /layered-upgrade-react\.js$/,
        layer: 'layered-components',
      },
      {
        test: /react\.js$/,
        issuerLayer: 'layered-components',
        layer: 'layered-components',
        use: [
          {
            loader: path.resolve(__dirname, './layered-react-loader.js'),
          },
        ],
      },
    ],
  },
};

/** @type {import("../../../../").Configuration[]} */
module.exports = [
  {
    ...commonConfig,
    output: {
      filename: '[name].js',
      uniqueName: '2-layers-full',
    },
    plugins: [
      new ModuleFederationPlugin({
        name: 'layers_container_2',
        library: { type: 'commonjs-module' },
        filename: 'container.js',
        remotes: {
          containerA: '../1-layers-full/container.js',
          containerB: './container.js',
        },
        ...commonMF,
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
      uniqueName: '2-layers-full-mjs',
    },
    plugins: [
      new ModuleFederationPlugin({
        name: 'layers_container_2',
        library: { type: 'module' },
        filename: 'module/container.mjs',
        remotes: {
          containerA: '../../1-layers-full/module/container.mjs',
          containerB: './container.mjs',
        },
        ...commonMF,
      }),
    ],
    target: 'node14',
  },
];
