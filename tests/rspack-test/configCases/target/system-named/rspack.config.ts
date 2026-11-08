import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    library: {
      name: 'named-system-module',
      type: 'system',
    },
  },
  node: {
    __dirname: false,
    __filename: false,
  },
});
