import { defineConfig } from '@rspack/cli';

export default defineConfig([
  {
    name: 'web',
    target: ['web', 'node'],
    output: {
      module: true,
      filename: 'web-[name].mjs',
    },
  },
]);
