import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: [`async-node${process.versions.node.split('.').map(Number)[0]}`],
  mode: 'none',
  experiments: {
    deferImport: true,
  },
  optimization: {
    moduleIds: 'named',
    chunkIds: 'named',
  },
});
