import { rspack } from '@rspack/core';
import TerserPlugin from 'terser-webpack-plugin';

/** @type {import("@rspack/core").Configuration} */
export default {
  node: {
    __dirname: false,
    __filename: false,
  },
  entry: {
    bundle0: ['./index.js'],
    'some-test': ['./test.js'],
  },
  output: {
    filename: '[name].js',
  },
  optimization: {
    minimizer: [new TerserPlugin()],
  },
  plugins: [
    new rspack.SourceMapDevToolPlugin({
      filename: 'sourcemaps/[file].map',
      append: (data) =>
        `\n//# sourceMappingURL=http://localhost:50505/[file].map`,
    }),
  ],
};
