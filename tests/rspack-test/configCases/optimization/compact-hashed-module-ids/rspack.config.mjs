import { rspack } from '@rspack/core';
if (
  rspack.ids.CompatHashedModuleIdsPlugin !==
  rspack.ids.CompactHashedModuleIdsPlugin
) {
  throw new Error(
    'CompatHashedModuleIdsPlugin must alias CompactHashedModuleIdsPlugin',
  );
}

/** @type {import("@rspack/core").Configuration[]} */
export default [
  {
    optimization: {
      moduleIds: 'compact-hashed',
    },
  },
  {
    entry: './min-length',
    optimization: {
      moduleIds: false,
    },
    plugins: [new rspack.ids.CompactHashedModuleIdsPlugin({ minLength: 1 })],
  },
  {
    optimization: {
      moduleIds: 'compat-hashed',
    },
  },
];
