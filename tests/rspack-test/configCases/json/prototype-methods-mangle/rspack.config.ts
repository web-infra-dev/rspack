import type { Configuration } from '@rspack/core';

export default [true, false].map((mangleExports) => ({
  mode: 'production',
  optimization: {
    mangleExports,
    minimize: false,
  },
})) satisfies Configuration[];
