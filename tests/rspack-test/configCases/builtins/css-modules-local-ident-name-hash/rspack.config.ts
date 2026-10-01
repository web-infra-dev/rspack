import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    path: 'node-commonjs path',
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/module',
        generator: {
          localIdentHashDigest: 'hex',
          localIdentHashDigestLength: 16,
          localIdentHashFunction: 'xxhash64',
          localIdentName: '[hash]',
        },
      },
    ],
  },
});
