import { rspack as webpack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration[]} */
export default [
  {
    optimization: {
      moduleIds: 'deterministic',
    },
  },
  {
    optimization: {
      moduleIds: false,
    },
    plugins: [
      new webpack.ids.DeterministicModuleIdsPlugin({
        maxLength: 0,
      }),
    ],
  },
  {
    optimization: {
      moduleIds: false,
    },
    plugins: [
      new webpack.ids.DeterministicModuleIdsPlugin({
        maxLength: 100,
      }),
    ],
  },
];
