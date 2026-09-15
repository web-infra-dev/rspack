const { experiments } = require('@rspack/core');

const { createPlugins, Layers } = experiments.rsc;
const { ServerPlugin, ClientPlugin } = createPlugins();

const swcLoaderRule = {
  test: /\.jsx?$/,
  use: [
    {
      loader: 'builtin:swc-loader',
      options: {
        detectSyntax: 'auto',
        rspackExperiments: {
          reactServerComponents: true,
        },
      },
    },
  ],
};

module.exports = [
  {
    mode: 'production',
    target: 'node',
    output: {
      filename: 'server-[name].js',
    },
    entry: {
      main: {
        import: './src/framework/entry.ssr.js',
      },
      rscService: {
        import: './src/framework/rsc-service.rsc.js',
      },
    },
    resolve: {
      extensions: ['...', '.jsx'],
    },
    module: {
      rules: [
        swcLoaderRule,
        {
          resource: /[\\/]framework[\\/].*\.ssr\.js$/,
          layer: Layers.ssr,
        },
        {
          resource: /[\\/]framework[\\/].*\.rsc\.js$/,
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
  },
  {
    mode: 'production',
    target: 'web',
    output: {
      filename: 'client-[name].js',
    },
    entry: {
      main: {
        import: './src/framework/entry.client.js',
      },
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
