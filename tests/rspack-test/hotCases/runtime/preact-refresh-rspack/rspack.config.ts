import { defineConfig } from '@rspack/cli';
import { PreactRefreshRspackPlugin } from '@rspack/plugin-preact-refresh';

export default defineConfig({
  plugins: [new PreactRefreshRspackPlugin()],
});
