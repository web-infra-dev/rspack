import { defineConfig } from '@rspack/cli';

export default defineConfig([
  {
    mode: 'development',
  },
  {
    mode: 'production',
  },
  {
    mode: 'production',
    optimization: {
      concatenateModules: false,
    },
  },
  {
    mode: 'development',
    optimization: {
      concatenateModules: true,
    },
  },
]);
