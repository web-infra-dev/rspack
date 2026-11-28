import { defineConfig } from '@rspack/cli';
import { container } from '@rspack/core';

export default defineConfig({
  externals: { fs: 'node-commonjs fs', path: 'node-commonjs path' },
  plugins: [
    new container.ModuleFederationPlugin({
      name: 'container',
      filename: 'container.js',
      library: { type: 'commonjs-module' },
      manifest: true,
      exposes: { './a': './a.js', './a-alias': './a.js' },
    }),
  ],
});
