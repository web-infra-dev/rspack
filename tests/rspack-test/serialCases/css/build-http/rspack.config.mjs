import path from 'node:path';

/** @type {import("@rspack/core").Configuration} */
export default {
  target: 'web',
  mode: 'development',
  experiments: {
    buildHttp: {
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
};
