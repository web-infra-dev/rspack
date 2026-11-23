import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: './index.js',
  ignoreWarnings: [
    {
      module: /module2\.js\?[34]/g,
    },
    {
      module: /[13]/g,
      message: /homepage/g,
    },
    /The 'mode' option has not been set/,
    (warning) => {
      return warning.module?.identifier().endsWith('?2') ?? false;
    },
  ],
  stats: {
    assets: true,
    modules: true,
  },
});
