import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externalsType: 'module-import',
  output: {
    pathinfo: true,
  },
});
