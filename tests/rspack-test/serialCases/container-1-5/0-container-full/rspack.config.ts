import { defineConfig } from '@rspack/cli';
import { container } from '@rspack/core';

const { ModuleFederationPlugin } = container;

const common = {
  name: 'container',
  exposes: {
    './ComponentA': {
      import: './ComponentA',
    },
  },
  shared: {
    '@rspack/mocked-react': {
      version: false as const,
      requiredVersion: false as const,
    },
  },
};

export default defineConfig([
  {
    output: {
      filename: '[name].js',
      uniqueName: '0-container-full',
    },
    plugins: [
      new ModuleFederationPlugin({
        library: { type: 'commonjs-module' },
        filename: 'container.js',
        remotes: {
          containerA: {
            external: './container.js',
          },
        },
        ...common,
      }),
    ],
  },
  {
    output: {
      module: true,
      filename: 'module/[name].mjs',
      uniqueName: '0-container-full-mjs',
    },
    plugins: [
      new ModuleFederationPlugin({
        library: { type: 'module' },
        filename: 'module/container.mjs',
        remotes: {
          containerA: {
            external: './container.mjs',
          },
        },
        ...common,
      }),
    ],
    target: 'node14',
  },
]);
