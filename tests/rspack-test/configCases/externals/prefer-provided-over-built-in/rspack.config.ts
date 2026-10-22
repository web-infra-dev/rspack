import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'node',
  externals: {
    http: '1+2',
  },
});
