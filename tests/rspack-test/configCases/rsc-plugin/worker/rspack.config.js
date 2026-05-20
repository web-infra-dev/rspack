const path = require('node:path');
const { experiments } = require('@rspack/core');

const { createPlugins, Layers } = experiments.rsc;
const { ServerPlugin, ClientPlugin } = createPlugins();

const ssrEntry = path.join(__dirname, 'src/framework/entry.ssr.js');
const rscEntry = path.join(__dirname, 'src/framework/entry.rsc.js');

const swcLoaderRule = {
  test: /\.jsx?$/,
  use: [
    {
      loader: 'builtin:swc-loader',
      options: {
        detectSyntax: 'auto',
        jsc: {
          transform: {
            react: {
              runtime: 'automatic',
            },
          },
        },
        rspackExperiments: {
          reactServerComponents: true,
        },
      },
    },
  ],
};

module.exports = [
  {
    name: 'server',
    mode: 'production',
    target: 'node',
    entry: {
      main: {
        import: ssrEntry,
      },
    },
    output: {
      filename: '[name].js',
    },
    resolve: {
      extensions: ['...', '.jsx'],
    },
    module: {
      rules: [
        swcLoaderRule,
        {
          resource: ssrEntry,
          layer: Layers.ssr,
        },
        {
          resource: rscEntry,
          layer: Layers.rsc,
          resolve: {
            conditionNames: ['react-server', '...'],
          },
        },
        {
          issuerLayer: Layers.rsc,
          resolve: {
            conditionNames: ['react-server', '...'],
          },
        },
      ],
    },
    plugins: [new ServerPlugin()],
    optimization: {
      splitChunks: {
        chunks: 'all',
        minSize: 0,
        cacheGroups: {
          vendor: {
            test: /[\\/]node_modules[\\/]/,
            name: 'vendor',
          },
        },
      },
    },
  },
  {
    name: 'client',
    mode: 'production',
    target: 'web',
    entry: {
      main: {
        import: './src/framework/entry.client.js',
      },
    },
    output: {
      filename: 'client.js',
    },
    resolve: {
      extensions: ['...', '.jsx'],
    },
    module: {
      rules: [swcLoaderRule],
    },
    plugins: [new ClientPlugin()],
  },
];
