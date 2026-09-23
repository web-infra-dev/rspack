import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: '../../basic/deferred-import/index.js',
  experiments: { deferImport: true },
  optimization: {
    splitChunks: {
      cacheGroups: {
        deferred: {
          test: /[\\/](dep|namespace|nested|shared)\.js$/,
          name: 'deferred',
          enforce: true,
        },
      },
    },
  },
});
