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
      return warning.module?.identifier().includes('b.js') ?? false;
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
