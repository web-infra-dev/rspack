const rspack = require('@rspack/core');

/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
  {
    optimization: {
      moduleIds: 'compat-hashed',
    },
  },
  {
    entry: './min-length',
    optimization: {
      moduleIds: false,
    },
    plugins: [new rspack.ids.CompatHashedModuleIdsPlugin({ minLength: 1 })],
  },
];
