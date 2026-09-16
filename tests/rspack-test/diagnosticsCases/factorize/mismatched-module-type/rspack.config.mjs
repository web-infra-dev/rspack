import path from 'node:path';

const resolve = (filename) => path.resolve(import.meta.dirname, filename);

/**
 * @type {import('@rspack/core').RspackOptions}
 */
export default {
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: resolve('app.jsx'),
        type: 'javascript/auto',
      },
      {
        test: resolve('app.tsx'),
        type: 'ts',
      },
    ],
  },
};
