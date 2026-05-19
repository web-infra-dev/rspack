import codspeedPlugin from '@codspeed/vitest-plugin';
import { defineConfig } from 'vitest/config';

export default defineConfig({
  plugins: [codspeedPlugin()],
  test: {
    fileParallelism: true,
    setupFiles: process.env.RSPACK_PGO_PROFILE_DUMP ? ['./pgo.setup.ts'] : [],
    poolOptions: {
      forks: {
        minForks: 1,
        maxForks: 8,
      },
    },
  },
});
