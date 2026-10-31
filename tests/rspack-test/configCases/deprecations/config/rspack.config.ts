import { defineConfig } from '@rspack/cli';

export default defineConfig({
  optimization: {
    // @ts-expect-error Preserve the deprecated webpack option tested by this fixture.
    noEmitOnErrors: true,
  },
});
