import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  context: import.meta.dirname,
  entry: './index.js',
  output: {
    chunkFilename: '[name].[contenthash].js',
    cssChunkFilename: '[name].[contenthash].css',
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
  plugins: [
    definePlugin((compiler) => {
      compiler.hooks.afterCompile.tap('StableWarnings', (compilation) => {
        compilation.warnings.sort((a, b) =>
          a.message === b.message ? 0 : a.message > b.message ? 1 : -1,
        );
      });
    }),
  ],
});
