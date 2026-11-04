import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
    'external-value': 'var { value: 1 }',
  },
  optimization: {
    moduleIds: 'named',
    concatenateModules: true,
    inlineExports: true,
  },
});
