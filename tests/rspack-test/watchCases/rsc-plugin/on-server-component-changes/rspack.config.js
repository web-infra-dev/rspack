const fs = require('node:fs');
const path = require('node:path');
const { experiments } = require('@rspack/core');

const { createPlugins, Layers } = experiments.rsc;
const { ServerPlugin, ClientPlugin } = createPlugins();

let serverCompiler;
let changeCount = 0;

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

const jsRule = {
  resolve: {
    extensions: ['...', '.jsx'],
  },
  module: {
    rules: [
      swcLoaderRule,
      {
        resource: /[\\/]framework[\\/]entry\.ssr\.js$/,
        layer: Layers.ssr,
      },
      {
        resource: /[\\/]framework[\\/]entry\.rsc\.js$/,
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
};

module.exports = [
  {
    ...jsRule,
    target: 'node',
    entry: {
      main: {
        import: './framework/entry.ssr.js',
      },
    },
    output: {
      filename: 'server-[name].js',
    },
    plugins: [
      function captureServerCompiler(compiler) {
        serverCompiler = compiler;
      },
      new ServerPlugin({
        onServerComponentChanges() {
          changeCount += 1;
          const logFile = path.join(
            serverCompiler.outputPath,
            'on-server-component-changes.log',
          );

          if (changeCount === 1) {
            fs.appendFileSync(logFile, 'callback returned void\n');
            return undefined;
          }

          return new Promise((resolve) => {
            setTimeout(() => {
              fs.appendFileSync(logFile, 'callback resolved promise\n');
              resolve();
            }, 50);
          });
        },
      }),
    ],
  },
  {
    ...jsRule,
    target: 'web',
    entry: {
      main: {
        import: './framework/entry.client.js',
      },
    },
    output: {
      filename: 'client-[name].js',
    },
    plugins: [new ClientPlugin()],
  },
];
