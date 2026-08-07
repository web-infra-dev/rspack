const rspack = require('@rspack/core');

/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
  {
    optimization: {
      moduleIds: 'compact',
    },
  },
  {
    entry: './min-length',
    optimization: {
      moduleIds: false,
    },
    plugins: [new rspack.ids.CompactModuleIdsPlugin({ minLength: 1 })],
  },
];
