import { defineConfig } from '@rspack/cli';
import { container } from '@rspack/core';

const { ContainerPlugin, ContainerReferencePlugin } = container;

const RUNTIME = 'container-runtime';

export default defineConfig({
  output: {
    filename: '[name].js',
  },
  plugins: [
    new ContainerPlugin({
      name: 'A',
      runtime: RUNTIME,
      filename: 'container-a.js',
      library: {
        type: 'commonjs-module',
      },
      exposes: {
        '.': './a',
      },
    }),
    new ContainerPlugin({
      name: 'B',
      runtime: RUNTIME,
      filename: 'container-b.js',
      library: {
        type: 'commonjs-module',
      },
      exposes: {
        '.': './b',
      },
    }),
    new ContainerReferencePlugin({
      remoteType: 'commonjs-module',
      remotes: {
        A: './container-a.js',
        B: './container-b.js',
      },
    }),
  ],
});
