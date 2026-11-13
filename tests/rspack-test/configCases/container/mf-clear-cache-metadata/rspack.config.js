const { ModuleFederationPlugin } = require('@rspack/core').container;

/** @type {import("@rspack/core").Configuration} */
const config = {
  target: 'node',
  entry: './index.js',
  output: {
    filename: 'bundle.js',
    uniqueName: 'mf-clear-cache-metadata',
  },
  optimization: {
    concatenateModules: false,
    chunkIds: 'named',
    moduleIds: 'named',
  },
  plugins: [
    new ModuleFederationPlugin({
      name: 'host',
      remotes: {
        remoteA: 'promise Promise.resolve({ init() {}, get() {} })',
      },
    }),
  ],
};

module.exports = [
  config,
  {
    ...config,
    target: 'web',
    entry: './browser.js',
    performance: false,
    output: { ...config.output, filename: 'browser.js' },
  },
];
