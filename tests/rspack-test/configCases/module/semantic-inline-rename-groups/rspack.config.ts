import { defineConfig } from '@rspack/cli';

export default defineConfig({
  optimization: {
    avoidEntryIife: true,
    concatenateModules: false,
    minimize: false,
  },
});
