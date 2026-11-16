import { defineConfig } from '@rspack/cli';

export default defineConfig([
  {
    optimization: {
      concatenateModules: false,
    },
  },
  {
    optimization: {
      concatenateModules: true,
    },
  },
]);
