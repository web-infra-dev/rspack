import codspeedPlugin from '@codspeed/vitest-plugin';
import { defineConfig } from 'vitest/config';

export default defineConfig({
  plugins: [codspeedPlugin()],
  test: {
    fileParallelism: true,
    poolOptions: {
      forks: {
        minForks: 1,
        maxForks: 8,
      },
    },
  },
});
