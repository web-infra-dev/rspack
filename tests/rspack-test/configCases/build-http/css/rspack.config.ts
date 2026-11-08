import { defineConfig } from '@rspack/cli';
import path from 'node:path';

export default defineConfig({
  externals: {
    path: 'node-commonjs path',
  },
  target: 'web',
  mode: 'development',
  output: {
    assetModuleFilename: '[hash][ext]',
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
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
  externalsPresets: {},
});
