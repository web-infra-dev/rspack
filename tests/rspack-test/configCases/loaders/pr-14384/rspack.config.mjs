import PluginWithLoader from './PluginWithLoader.js';

/** @type {import("@rspack/core").Configuration} */
export default {
  plugins: [new PluginWithLoader()],
};
