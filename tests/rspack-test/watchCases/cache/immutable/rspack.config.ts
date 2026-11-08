import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  cache: true,
  output: {
    filename: 'bundle.js?[contenthash]',
  },
});
