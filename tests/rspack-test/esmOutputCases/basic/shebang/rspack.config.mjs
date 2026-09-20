import { rspack } from '@rspack/core';

export default {
  optimization: {
    splitChunks: {
      cacheGroups: {
        splitMain: {
          test: /index\.js/,
        },
      },
    },
  },
  plugins: [new rspack.experiments.RslibPlugin()],
};
