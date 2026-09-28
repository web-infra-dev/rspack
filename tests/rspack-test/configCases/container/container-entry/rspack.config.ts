import { defineConfig } from '@rspack/cli';
import { container } from '@rspack/core';

const { ContainerPlugin } = container;

export default defineConfig({
  externals: {
    './container-file.js': 'commonjs ./container-file.js',
  },
  output: {
    pathinfo: true,
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
        './test2': ['./init-module', './test2'],
        '.': './main',
      },
    }),
  ],
});
