import { defineConfig } from '@rspack/cli';
import { container, sharing } from '@rspack/core';

const { ContainerPlugin } = container;
const { ConsumeSharedPlugin } = sharing;

export default defineConfig({
  externals: {
    './container-file.js': 'commonjs ./container-file.js',
  },
  plugins: [
    new ContainerPlugin({
      name: 'container',
      filename: 'container-file.js',
      library: {
        type: 'commonjs-module',
      },
      exposes: {
        './test': './test',
      },
    }),
    new ConsumeSharedPlugin({
      consumes: {
        './value': {
          shareKey: 'value',
        },
      },
    }),
  ],
});
