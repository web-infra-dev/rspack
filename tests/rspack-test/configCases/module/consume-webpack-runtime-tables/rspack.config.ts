import { defineConfig } from '@rspack/cli';

export default defineConfig(
  [false, true].flatMap((concatenateModules) =>
    [false, true].map((minimize) => ({
      output: {
        module: true,
        chunkFormat: 'module',
      },
      optimization: {
        avoidEntryIife: true,
        concatenateModules,
        minimize,
      },
    })),
  ),
);
