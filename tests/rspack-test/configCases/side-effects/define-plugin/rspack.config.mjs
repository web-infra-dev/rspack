import { rspack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  externals: {
    fs: 'node-commonjs fs',
  },
  optimization: {
    sideEffects: true,
    moduleIds: 'named',
    concatenateModules: false,
  },
  plugins: [
    new rspack.DefinePlugin({
      'process.env.NODE_ENV': JSON.stringify('production'),
    }),
  ],
};
