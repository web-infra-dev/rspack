import { defineConfig } from '../../../dist/index.js';

export default defineConfig({
  mode: 'production',
  devServer: {
    port: Number(process.env.RSPACK_PREVIEW_TEST_PORT),
  },
});
