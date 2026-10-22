import { defineConfig } from '@rspack/cli';
import { container } from '@rspack/core';

const { ModuleFederationPlugin } = container;

export default defineConfig({
  plugins: [
    new ModuleFederationPlugin({
      remoteType: 'var',
      remotes: {
        abc: 'ABC',
        def: 'DEF',
      },
    }),
  ],
});
