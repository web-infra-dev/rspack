import assert from 'node:assert/strict';
import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: './index',
  stats: 'errors-warnings',
  ignoreWarnings: [
    /Using \/ for division outside/,
    {
      message: /ESModulesLinkingWarning/,
    },
    {
      module: /a.js/,
    },
    (warning) => {
      assert(warning.module);
      return warning.module.identifier().includes('b.js');
    },
  ],
  module: {
    parser: {
      javascript: {
        exportsPresence: 'auto',
      },
    },
    rules: [
      {
        test: /\.s[ac]ss$/i,
        use: [{ loader: 'sass-loader' }],
        type: 'css',
      },
    ],
  },
});
