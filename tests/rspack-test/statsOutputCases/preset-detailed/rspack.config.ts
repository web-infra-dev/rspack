import { defineConfig } from '@rspack/cli';
import { LogTestPlugin } from '@rspack/test-tools/helper/legacy/LogTestPlugin';

export default defineConfig({
  incremental: false,
  mode: 'production',
  entry: './index',
  stats: 'detailed',
  infrastructureLogging: {
    level: 'log',
  },
  plugins: [new LogTestPlugin()],
});
