import { defineConfig } from '@rspack/cli';
import { LogTestPlugin } from '@rspack/test-tools/helper/legacy/LogTestPlugin';

export default defineConfig({
  mode: 'production',
  entry: './index',
  stats: 'errors-only',
  infrastructureLogging: {
    level: 'error',
  },
  plugins: [new LogTestPlugin()],
});
