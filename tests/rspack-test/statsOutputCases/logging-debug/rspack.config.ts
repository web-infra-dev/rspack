import { defineConfig } from '@rspack/cli';
import { LogTestPlugin } from '@rspack/test-tools/helper/legacy/LogTestPlugin';
import { fileURLToPath } from 'node:url';

export default defineConfig({
  mode: 'production',
  entry: './index',
  performance: false,
  module: {
    rules: [
      {
        test: /index\.js$/,
        use: fileURLToPath(
          import.meta.resolve('../logging/node_modules/custom-loader/index.js'),
        ),
      },
    ],
  },
  plugins: [new LogTestPlugin(true)],
  stats: {
    assets: true,
    modules: true,
    colors: true,
    logging: false,
    loggingDebug: /custom-loader/,
  },
});
