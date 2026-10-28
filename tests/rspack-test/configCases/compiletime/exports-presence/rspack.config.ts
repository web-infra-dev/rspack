import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  module: {
    rules: [
      {
        test: /aaa/,
        parser: {
          exportsPresence: false,
        },
      },
      {
        test: /bbb/,
        parser: {
          exportsPresence: 'warn',
        },
      },
      {
        test: /ccc/,
        parser: {
          exportsPresence: 'error',
        },
      },
      {
        test: /ddd/,
        parser: {
          exportsPresence: 'error',
          importExportsPresence: 'warn',
          reexportExportsPresence: false,
        },
      },
    ],
  },
});
