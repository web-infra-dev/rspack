import path from 'node:path';

/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    main: './index.js',
  },
  resolve: {
    tsConfig: {
      configFile: path.resolve(import.meta.dirname, './tsconfig.json'),
      references: 'auto',
    },
  },
};
