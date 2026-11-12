import { defineConfig } from '@rspack/cli';

export default defineConfig([
  {
    target: 'web',
    experiments: {
      futureDefaults: true,
    },
  },
  {
    target: 'web',
    node: {
      __filename: 'mock',
      __dirname: 'mock',
      global: 'warn',
    },
  },
  {
    target: 'web',
    node: {
      __filename: 'warn-mock',
      __dirname: 'warn-mock',
      global: true,
    },
  },
]);
