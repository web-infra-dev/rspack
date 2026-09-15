import { defineConfig } from '@rspack/cli';
import type { MultiRspackOptions, RspackOptions } from '@rspack/core';

defineConfig({
  mode: 'development',
});

defineConfig([
  {
    mode: 'development',
  },
  {
    mode: 'production',
  },
]);

defineConfig(() => ({
  mode: 'development',
}));

defineConfig((env, argv) => ({
  mode:
    env.production || argv.mode === 'production' ? 'production' : 'development',
}));

defineConfig(() => [
  {
    mode: 'development',
  },
  {
    mode: 'production',
  },
]);

defineConfig(async () => ({
  mode: 'development',
}));

defineConfig(async () => [
  {
    mode: 'development',
  },
  {
    mode: 'production',
  },
]);

defineConfig({
  // @ts-expect-error invalid mode
  mode: 'invalid',
});

// @ts-expect-error invalid mode
defineConfig(() => ({
  mode: 'invalid',
}));

// @ts-expect-error invalid mode
defineConfig(async () => ({
  mode: 'invalid',
}));

const single: RspackOptions = defineConfig({
  mode: 'development',
});

const multi: MultiRspackOptions = defineConfig([
  {
    mode: 'development',
  },
]);

const syncResult: RspackOptions | MultiRspackOptions = defineConfig(() => ({
  mode: 'development',
}))({}, {});

const asyncResult: Promise<RspackOptions | MultiRspackOptions> = defineConfig(
  async () => ({
    mode: 'development',
  }),
)({}, {});

declare const dynamicConfig: RspackOptions | MultiRspackOptions;
defineConfig(dynamicConfig);

export { single, multi, syncResult, asyncResult };
