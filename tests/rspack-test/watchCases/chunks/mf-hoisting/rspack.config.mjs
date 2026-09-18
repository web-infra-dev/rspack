import { rspack } from '@rspack/core';
import { ReactRefreshRspackPlugin } from '@rspack/plugin-react-refresh';

/** @type {import("@rspack/core").Configuration} */
export default {
  target: 'web',
  plugins: [
    new rspack.container.ModuleFederationPlugin({
      name: 'test',
      shareStrategy: 'loaded-first',
    }),
    new ReactRefreshRspackPlugin(), // Need this to trigger hoisting (hoist_container_references_plugin.rs)
  ],
};
