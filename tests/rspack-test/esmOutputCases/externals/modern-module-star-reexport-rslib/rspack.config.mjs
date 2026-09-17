import { rspack } from '@rspack/core';

const {
  experiments: { RslibPlugin },
} = rspack;

export default {
  externalsType: 'modern-module',
  externals: {
    externals: 'externals',
  },
  plugins: [new RslibPlugin()],
};
