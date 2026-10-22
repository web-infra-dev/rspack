import { defineConfig } from '@rspack/cli';

export default defineConfig({
  optimization: {
    moduleIds: 'named',
  },
  output: {
    webassemblyModuleFilename: '[id].[hash].wasm',
  },
  experiments: {
    asyncWebAssembly: true,
  },
});
