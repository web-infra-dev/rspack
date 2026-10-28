import { defineConfig } from '@rspack/cli';
import { LogTestPlugin } from '@rspack/test-tools/helper/legacy/LogTestPlugin';

export default defineConfig({
  mode: 'production',
  entry: './index',
  stats: 'errors-warnings',
  infrastructureLogging: {
    level: 'warn',
  },
  plugins: [new LogTestPlugin()],
});
