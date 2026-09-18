import path from 'node:path';

/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    main: './index.js',
  },
  resolve: {
    tsConfig: path.resolve(import.meta.dirname, './tsconfig.json'),
  },
};
