const rspack = require('@rspack/core');

/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
  {
    optimization: {
      moduleIds: 'compact',
    },
  },
  {
    optimization: {
      moduleIds: false,
    },
    plugins: [new rspack.ids.CompactModuleIdsPlugin()],
  },
];
