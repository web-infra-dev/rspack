import { rspack } from '@rspack/core';
export default {
  mode: 'production',
  plugins: [
    new rspack.DefinePlugin({
      FALSY: JSON.stringify(false),
    }),
  ],
};
