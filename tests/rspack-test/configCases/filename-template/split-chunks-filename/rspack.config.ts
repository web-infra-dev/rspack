import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  node: {
    __dirname: false,
    __filename: false,
  },
  output: {
    library: { type: 'commonjs2' },
  },
  optimization: {
    splitChunks: {
      cacheGroups: {
        vendor: {
          test: /node_modules/,
          chunks: 'initial',
          filename: 'vendor.js',
          enforce: true,
        },
      },
    },
  },
});
