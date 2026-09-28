import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    fs: 'module fs',
    path: 'module path',
  },
  externalsType: 'module-import',
});
