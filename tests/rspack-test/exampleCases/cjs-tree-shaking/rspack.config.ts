import { defineConfig } from '@rspack/cli';

export default defineConfig([
  {
    entry: './example.js',
    output: {
      pathinfo: true,
      filename: 'output.js',
    },
    optimization: {
      // @ts-expect-error Rspack does not support size module IDs yet.
      moduleIds: 'size',
      usedExports: true,
      mangleExports: true,
    },
  },
  {
    entry: './example.js',
    output: {
      pathinfo: true,
      filename: 'without.js',
    },
    optimization: {
      // @ts-expect-error Rspack does not support size module IDs yet.
      moduleIds: 'size',
      usedExports: false,
      mangleExports: false,
    },
  },
]);
