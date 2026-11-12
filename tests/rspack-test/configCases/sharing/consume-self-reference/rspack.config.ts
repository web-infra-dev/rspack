import { defineConfig } from '@rspack/cli';
import { sharing } from '@rspack/core';

const { SharePlugin } = sharing;

export default defineConfig({
  plugins: [
    new SharePlugin({
      shared: {
        'my-middleware': {
          singleton: true,
          // import: false
        },
        'my-module/a': {
          singleton: true,
          version: '1.2.3',
          // import: false
        },
        'my-module/b': {
          singleton: true,
          version: '1.2.3',
          // import: false
        },
      },
    }),
  ],
});
