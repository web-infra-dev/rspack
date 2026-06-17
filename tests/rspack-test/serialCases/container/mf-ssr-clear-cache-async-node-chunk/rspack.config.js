const { ModuleFederationPlugin } = require('@rspack/core').container;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: 'async-node',
  entry: {
    main: './index.js',
  },
  output: {
    chunkLoading: 'async-node',
    filename: '[name].js',
    uniqueName: 'mf-ssr-clear-cache-async-node-chunk',
  },
  plugins: [
    new ModuleFederationPlugin({
      name: 'host_async_chunk',
      remotes: {
        remoteA:
          'promise Promise.resolve(globalThis.__mfSsrClearCacheAsyncChunkRemoteServer.loadRemoteEntry())',
      },
    }),
  ],
};
