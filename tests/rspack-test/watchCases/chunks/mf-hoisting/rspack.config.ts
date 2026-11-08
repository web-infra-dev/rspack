import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';
import { ReactRefreshRspackPlugin } from '@rspack/plugin-react-refresh';

export default defineConfig({
  target: 'web',
  plugins: [
    new rspack.container.ModuleFederationPlugin({
      name: 'test',
      shareStrategy: 'loaded-first',
    }),
    new ReactRefreshRspackPlugin(), // Need this to trigger hoisting (hoist_container_references_plugin.rs)
  ],
});
