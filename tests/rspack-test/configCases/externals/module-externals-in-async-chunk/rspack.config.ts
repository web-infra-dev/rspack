import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    external: 'fs',
    external2: 'node:fs',
    external3: 'fs',
  },
  externalsType: 'module-import',
  output: {
    module: true,
    chunkFormat: 'module',
    chunkFilename: '[name].mjs',
  },
  optimization: {
    moduleIds: 'named',
    concatenateModules: false,
  },
});
