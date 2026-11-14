import { defineConfig } from '@rspack/cli';
import { container, HotModuleReplacementPlugin } from '@rspack/core';

export default defineConfig({
  target: 'node',
  externals: { './container-file.js': 'commonjs ./container-file.js' },
  plugins: [
    new HotModuleReplacementPlugin(),
    new container.ModuleFederationPlugin({
      name: 'hmr_container',
      filename: 'container-file.js',
      library: { type: 'commonjs-module' },
      exposes: { './value': './value' },
    }),
  ],
});
