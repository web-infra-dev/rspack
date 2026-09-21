import { defineConfig } from '@rspack/cli';

export default defineConfig(
  [true, false].map((concatenateModules) => {
    return {
      target: 'node',
      optimization: {
        concatenateModules,
      },
    };
  }),
);
