import { defineConfig } from '@rspack/cli';
import path from 'node:path';

export default defineConfig({
  target: 'web',
  mode: 'development',
  experiments: {
    buildHttp: {
      // @ts-expect-error Function allowedUris is not supported by Rspack yet.
      allowedUris: [() => true],
      lockfileLocation: path.resolve(
        import.meta.dirname,
        './lock-files/lock.json',
      ),
      cacheLocation: path.resolve(import.meta.dirname, './lock-files/test'),
    },
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
});
