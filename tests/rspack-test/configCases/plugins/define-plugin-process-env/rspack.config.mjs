import { DefinePlugin } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  target: 'node',
  mode: 'production',
  plugins: [
    new DefinePlugin({
      'process.env.ENVIRONMENT': JSON.stringify('node'),
    }),
  ],
};
