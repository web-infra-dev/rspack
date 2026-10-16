import { defineConfig } from '@rspack/cli';

export default defineConfig({
  experiments: {
    deferImport: true,
  },
  module: {
    parser: {
      javascript: {
        deferImport: false,
      },
    },
    rules: [
      {
        oneOf: [
          {
            test: /consumer\.js$/,
            rules: [
              {
                parser: {
                  deferImport: true,
                },
              },
            ],
          },
        ],
      },
    ],
  },
});
