import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';
import TerserPlugin from 'terser-webpack-plugin';

export default defineConfig({
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
      append: () => `\n//# sourceMappingURL=http://localhost:50505/[file].map`,
    }),
  ],
});
