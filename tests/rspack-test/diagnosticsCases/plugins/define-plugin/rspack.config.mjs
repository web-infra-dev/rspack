import { rspack } from '@rspack/core';
export default {
  optimization: {
    nodeEnv: 'development',
  },
  plugins: [
    new rspack.DefinePlugin({
      'process.env.NODE_ENV': JSON.stringify('production'),
    }),
  ],
};
