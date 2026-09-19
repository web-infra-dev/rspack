import { defineConfig } from '@rspack/cli';

import { LogTestPlugin } from '@rspack/test-tools/helper/legacy/LogTestPlugin';

export default defineConfig({
  incremental: false,
  mode: 'production',
  entry: './index',
  performance: false,
  module: {
    rules: [
      {
        test: /index\.js$/,
        use: 'custom-loader',
      },
    ],
  },
  plugins: [new LogTestPlugin(true)],
  stats: {
    assets: true,
    modules: true,
    colors: true,
    logging: true,
    loggingDebug: 'custom-loader',
    loggingTrace: true,
  },
});
