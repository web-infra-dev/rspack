import { rspack } from '@rspack/core';

/** @type {import('@rspack/core').Configuration} */
export default {
  plugins: [
    new rspack.DefinePlugin({
      DEFINE_VAR: '1 2 3',
    }),
  ],
  optimization: {
    concatenateModules: true,
  },
};
