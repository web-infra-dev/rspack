import { defineConfig } from '@rspack/cli';
import { LogTestPlugin } from '@rspack/test-tools/helper/legacy/LogTestPlugin';

export default defineConfig({
  incremental: false,
  mode: 'production',
  entry: './index',
  profile: true,
  stats: 'verbose',
  infrastructureLogging: {
    level: 'verbose',
  },
  plugins: [new LogTestPlugin()],
});
