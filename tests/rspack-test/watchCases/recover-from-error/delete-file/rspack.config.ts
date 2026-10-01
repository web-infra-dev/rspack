import { defineConfig } from '@rspack/cli';

export default defineConfig({
  ignoreWarnings: [/FlagDependencyUsagePlugin/],
  optimization: {
    usedExports: true,
  },
});
