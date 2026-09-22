import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

const {
  experiments: { RslibPlugin },
} = rspack;

export default defineConfig({
  target: 'node',
  experiments: {
    runtimeMode: 'rspack',
  },
  module: {
    rules: [
      {
        test: /\.svg$/,
        type: 'asset/resource',
      },
    ],
  },
  output: {
    iife: false,
    library: {
      type: 'commonjs-static',
    },
  },
  plugins: [new RslibPlugin()],
});
