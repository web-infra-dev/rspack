import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    module: true,
    filename: '[name].mjs',
    library: { type: 'module' },
  },
  optimization: {
    runtimeChunk: 'single', // any value other than `false`
  },
});
