import path from 'node:path';

/** @type {import("@rspack/core").Configuration} */
export default {
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
};
