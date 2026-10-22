import { defineConfig } from '@rspack/cli';

export default defineConfig({
  optimization: {
    moduleIds: 'named',
    usedExports: true,
    providedExports: true,
    concatenateModules: true,
  },
});
