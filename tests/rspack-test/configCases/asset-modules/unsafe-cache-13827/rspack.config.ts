import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  module: {
    rules: [
      {
        dependency: 'url',
        type: 'asset',
      },
    ],
  },
});
