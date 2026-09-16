import { rspack } from '@rspack/core';

const {
  experiments: { RslibPlugin },
} = rspack;

export default {
  externals: {
    fs: 'commonjs fs',
  },
  plugins: [new RslibPlugin()],
};
