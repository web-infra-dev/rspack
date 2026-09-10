import { rspack } from '@rspack/core';

export default {
  target: 'node',
  mode: 'development',
  plugins: [
    new rspack.DefinePlugin({
      EXPORT_KIND: JSON.stringify('esm-default-object'),
    }),
  ],
};
