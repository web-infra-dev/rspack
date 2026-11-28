import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'node',
  mode: 'development',
  devtool: false,
  output: {
    module: true,
    filename: 'bundle0.mjs',
    environment: { topLevelAwait: false },
  },
});
