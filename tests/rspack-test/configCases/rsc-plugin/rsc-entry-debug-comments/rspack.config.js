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

const cssRule = {
  test: /\.css$/,
  type: 'css/auto',
};

module.exports = [
  {
    mode: 'production',
    target: 'node',
    entry: {
      main: {
        import: ssrEntry,
      },
    },
    output: {
      filename: 'server.js',
      pathinfo: false,
    },
    resolve: {
      extensions: ['...', '.jsx'],
    },
    module: {
      rules: [
        cssRule,
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
    node: {
      __dirname: false,
      __filename: false,
    },
    plugins: [new ServerPlugin()],
    optimization: {
      moduleIds: 'named',
      chunkIds: 'named',
    },
  },
  {
    mode: 'production',
    target: 'web',
    entry: {
      main: {
        import: './src/framework/entry.client.js',
      },
    },
    output: {
      filename: 'client.js',
      pathinfo: false,
    },
    resolve: {
      extensions: ['...', '.jsx'],
    },
    module: {
      rules: [cssRule, swcLoaderRule],
    },
    plugins: [new ClientPlugin()],
    optimization: {
      moduleIds: 'named',
      chunkIds: 'named',
    },
  },
];
