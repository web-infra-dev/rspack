import { rspack } from '@rspack/core';

const {
  experiments: { RslibPlugin },
} = rspack;

export default {
  externals: {
    fs: 'module fs',
    path: 'module path',
  },
  plugins: [new RslibPlugin()],
};
