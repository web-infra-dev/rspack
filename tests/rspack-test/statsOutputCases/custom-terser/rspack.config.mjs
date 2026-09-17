import TerserPlugin from 'terser-webpack-plugin';

/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'production',
  entry: './index',
  output: {
    filename: 'bundle.js',
  },
  optimization: {
    minimize: true,
    minimizer: [
      new TerserPlugin({
        terserOptions: {
          mangle: false,
          output: {
            beautify: true,
            comments: false,
          },
        },
      }),
    ],
  },
  stats: {
    assets: true,
    chunkModules: false,
    modules: true,
    providedExports: true,
    usedExports: true,
  },
};
