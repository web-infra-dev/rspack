import PluginWithLoader from './PluginWithLoader.mjs';

/** @type {import("@rspack/core").Configuration} */
export default {
  plugins: [new PluginWithLoader()],
};
