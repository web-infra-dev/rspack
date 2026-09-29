import { defineConfig } from '@rspack/cli';

export default defineConfig({
  // mode: "development" || "production",
  optimization: {
    concatenateModules: false,
  },
});
