import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  entry: './index',
  stats: {
    all: false,
    modules: true,
  },
  plugins: [
    new rspack.IgnorePlugin({
      checkResource: (resource) => {
        if (resource.includes('zh') || resource.includes('globalIndex')) {
          return true;
        }
        return false;
      },
    }),
  ],
});
