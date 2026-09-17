import { DefinePlugin } from '@rspack/core';

/**@type {import("@rspack/core").Configuration}*/
export default {
  context: import.meta.dirname,

  optimization: {
    sideEffects: true,
  },
  plugins: [
    new DefinePlugin({
      'process.env.NODE_ENV': JSON.stringify('production'),
    }),
  ],
};
