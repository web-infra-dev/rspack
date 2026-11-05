import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    // @ts-expect-error Preserve the unsupported webpack context options in this fixture.
    unknownContextRegExp: /^\.\//,
    unknownContextCritical: false,
    exprContextRegExp: /^\.\//,
    exprContextCritical: false,
  },
});
