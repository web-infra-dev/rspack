import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    importFunctionName: 'import.meta.__customImport__',
  },
  externals: {
    os: 'commonjs os',
  },
});
