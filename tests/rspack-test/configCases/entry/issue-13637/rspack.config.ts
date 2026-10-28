import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    'main-system': {
      import: './index-system.js',
      library: {
        type: 'system',
      },
      filename: 'main.system.js',
    },
    'main-umd': {
      import: './index-umd.js',
      library: {
        type: 'umd',
      },
      filename: 'main.umd.js',
    },
  },
  node: {
    __dirname: false,
    __filename: false,
  },
});
