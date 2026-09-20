import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  externals: {
    'node:fs': 'node-commonjs node:fs',
    'node:path': 'node-commonjs node:path',
  },
  plugins: [
    new rspack.DefinePlugin({
      'process.env.test': {
        NODE_ENV: '"development"',
        PUBLIC_URL: '""',
        WDS_SOCKET_HOST: undefined,
        WDS_SOCKET_PATH: undefined,
        WDS_SOCKET_PORT: undefined,
        FAST_REFRESH: 'true',
      },
    }),
  ],
});
