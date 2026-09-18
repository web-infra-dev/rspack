import { rspack as webpack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  optimization: {
    moduleIds: false,
  },
  plugins: [new webpack.ids.HashedModuleIdsPlugin()],
};
