import { defineConfig } from '@rspack/cli';

export default defineConfig(
  [false, true].map((split, index) => ({
    mode: 'development',
    target: 'node',
    amd: {},
    output: {
      filename: `${index}/[name].js`,
      chunkFilename: `${index}/[name].js`,
    },
    optimization: {
      runtimeChunk: split ? 'single' : false,
      splitChunks: split
        ? {
            chunks: 'all',
            cacheGroups: {
              modules: {
                test: /(?:define|require)-.*\.js$/,
                name: 'modules',
                enforce: true,
              },
            },
          }
        : false,
    },
  })),
);
