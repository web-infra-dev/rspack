import { defineConfig } from '@rspack/cli';

export default defineConfig(
  [false, true].flatMap((minimize) =>
    ['./index.js', './cache.js'].map((entry) => ({
      entry,
      optimization: {
        avoidEntryIife: true,
        concatenateModules: false,
        moduleIds: 'named',
        minimize,
      },
    })),
  ),
);
