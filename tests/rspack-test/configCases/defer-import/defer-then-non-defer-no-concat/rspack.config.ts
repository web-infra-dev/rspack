import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: [`async-node${process.versions.node.split('.').map(Number)[0]}`],
  mode: 'production',
  optimization: {
    concatenateModules: false,
  },
  experiments: {
    deferImport: true,
  },
});
