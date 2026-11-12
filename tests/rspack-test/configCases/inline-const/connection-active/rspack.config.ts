import { defineConfig } from '@rspack/cli';

export default defineConfig({
  optimization: {
    moduleIds: 'named',
    inlineExports: true,
  },
});
