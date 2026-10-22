import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: [`async-node${process.versions.node.split('.').map(Number)[0]}`],
  entry: ['../defer-runtime/all-dynamic-import.js'],
  optimization: {
    concatenateModules: false,
  },
  experiments: {
    deferImport: true,
  },
});
