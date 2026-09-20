import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    main: './entry-a.js',
    other: './entry-b.js',
  },
  optimization: {
    runtimeChunk: {
      name: (entrypoint) => `runtime-${entrypoint.name}`,
    },
  },
});
