import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'node',
  output: {
    module: true,
    importMetaName: 'custom',
  },
  node: {
    __filename: 'node-module',
    __dirname: 'node-module',
  },
});
