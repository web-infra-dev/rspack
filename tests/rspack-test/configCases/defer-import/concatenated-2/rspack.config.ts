import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: [`async-node${process.versions.node.split('.').map(Number)[0]}`],
  optimization: {
    concatenateModules: true,
  },
  experiments: {
    deferImport: true,
  },
});
