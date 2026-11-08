import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  optimization: {
    nodeEnv: 'development',
  },
});
