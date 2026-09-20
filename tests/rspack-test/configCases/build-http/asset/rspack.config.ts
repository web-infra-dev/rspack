import { defineConfig } from '@rspack/cli';
import path from 'node:path';

export default defineConfig({
  target: 'web',
  mode: 'development',
  module: {
    rules: [
      {
        test: /\.png$/,
        type: 'asset/resource',
      },
    ],
  },
  experiments: {
    buildHttp: {
      allowedUris: ['https://raw.githubusercontent.com/'],
      lockfileLocation: path.resolve(
        import.meta.dirname,
        './lock-files/lock.json',
      ),
      cacheLocation: path.resolve(import.meta.dirname, './lock-files/test'),
    },
  },
});
