import path from 'node:path';

/**@type {import("@rspack/core").Configuration}*/
export default {
  mode: 'production',
  context: import.meta.dirname,
  entry: {
    main: './index.js',
    sub: './sub.js',
  },
  output: {
    filename: '[name].js',
  },
  optimization: {
    minimize: false,
    mangleExports: false,
    moduleIds: 'named',
    usedExports: 'global',
  },
};
