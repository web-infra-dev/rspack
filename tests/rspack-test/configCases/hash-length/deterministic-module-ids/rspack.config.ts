import { defineConfig } from '@rspack/cli';
import { rspack as webpack } from '@rspack/core';

export default defineConfig([
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
]);
