import { defineConfig } from '@rspack/cli';
import { container } from '@rspack/core';

const { ModuleFederationPlugin } = container;

const common = defineConfig({
  entry: {
    main: {
      import: './index.js',
      runtime: 'other',
    },
    another: {
      import: './index.js',
      runtime: 'webpack',
    },
  },
  optimization: {
    runtimeChunk: 'single',
  },
});

const commonMF = {
  runtime: false as const,
  exposes: {
    './ComponentB': './ComponentB',
    './ComponentC': './ComponentC',
  },
  shared: ['react'],
};

export default defineConfig([
  {
    mode: 'production',
    ...common,
    output: {
      filename: '[name].js',
      uniqueName: '1-container-full',
    },
    plugins: [
      new ModuleFederationPlugin({
        name: 'container',
        library: { type: 'commonjs-module' },
        filename: 'container.js',
        remotes: {
          containerA: '../0-container-full/container.js',
          containerB: './container.js',
        },
        ...commonMF,
      }),
    ],
  },
  {
    ...common,
    output: {
      module: true,
      filename: 'module/[name].mjs',
      uniqueName: '1-container-full-mjs',
    },
    plugins: [
      new ModuleFederationPlugin({
        name: 'container',
        library: { type: 'module' },
        filename: 'module/container.mjs',
        remotes: {
          containerA: '../../0-container-full/module/container.mjs',
          containerB: './container.mjs',
        },
        ...commonMF,
      }),
    ],
    target: 'node14',
  },
]);
