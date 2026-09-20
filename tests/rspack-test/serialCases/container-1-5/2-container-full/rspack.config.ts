import { defineConfig } from '@rspack/cli';

import { container } from '@rspack/core';

const { ModuleFederationPlugin } = container;

export default defineConfig({
  output: {
    uniqueName: '2-container-full',
  },
  plugins: [
    new ModuleFederationPlugin({
      name: 'main',
      library: { type: 'commonjs-module' },
      remotes: {
        containerB: '../1-container-full/container.js',
        self: [
          'var undefined',
          'var (() => { throw new Error(); })()',
          'var { then: (a, b) => b(new Error()) }',
          './bundle0.js',
        ],
      },
      exposes: ['./Self'],
      shared: {
        '@rspack/mocked-react': '@rspack/mocked-react',
        'old-react': {
          import: false,
          shareKey: '@rspack/mocked-react',
          requiredVersion: '^2',
        },
        'old-react-singleton': {
          import: false,
          shareKey: '@rspack/mocked-react',
          requiredVersion: '^2',
          singleton: true,
        },
      },
    }),
  ],
});
