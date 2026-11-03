import { defineConfig } from '@rspack/cli';
import type {
  CssParserOptions,
  MultiRspackOptions,
  RspackOptions,
} from '@rspack/core';

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

const cssRuntimePublicPath: RspackOptions = {
  module: {
    parser: {
      css: { exportType: 'style' },
      'css/auto': { exportType: 'style' },
      'css/global': { exportType: 'style' },
      'css/module': { exportType: 'style' },
    },
    generator: {
      css: { runtimePublicPath: true },
      'css/auto': { runtimePublicPath: true },
      'css/global': { runtimePublicPath: false },
      'css/module': { runtimePublicPath: true },
    },
  },
};
defineConfig(cssRuntimePublicPath);

const cssParserOptions: CssParserOptions = {
  // @ts-expect-error runtimePublicPath is a generator option
  runtimePublicPath: true,
};
defineConfig({ module: { parser: { css: cssParserOptions } } });
