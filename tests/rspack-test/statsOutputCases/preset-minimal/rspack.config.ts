import { defineConfig } from '@rspack/cli';
import { LogTestPlugin } from '@rspack/test-tools/helper/legacy/LogTestPlugin';

export default defineConfig({
  mode: 'production',
  entry: './index',
  stats: 'minimal',
  infrastructureLogging: {
    level: 'warn',
  },
  plugins: [new LogTestPlugin()],
});
