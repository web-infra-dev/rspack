import rspack, { type RspackOptions } from '@rspack/core';
import { defineConfig, definePlugin, type Configuration } from '@rspack/cli';

const plugin = definePlugin({
  apply(compiler) {
    compiler.hooks.done.tap('type-test', () => undefined);
  },
});

const config: RspackOptions = {
  entry: './src/index.js',
  plugins: [
    plugin,
    new rspack.DefinePlugin({
      __TYPE_TEST__: JSON.stringify(true),
    }),
  ],
  devServer: {
    proxy: [
      {
        context: ['/api'],
        target: 'http://localhost:3000',
      },
    ],
  },
};

export const cliConfig: Configuration = defineConfig(config);

const observeWatchOrigin = (compiler: import('@rspack/core').Compiler) => {
  compiler.hooks.watchInvalidation.tap('type-test', (event) => {
    const invalidation: import('@rspack/core').WatchInvalidation = event;
    const cause: import('@rspack/core').WatchCause = invalidation.cause;
    if (cause.kind === 'source') {
      const paths: readonly string[] = cause.changed;
      void paths;
    }
    // @ts-expect-error Watch invalidations are immutable.
    invalidation.revision = 1;
  });
  compiler.hooks.done.tap('type-test', (stats) => {
    const origin: import('@rspack/core').RebuildOrigin | undefined =
      stats.compilation.rebuildOrigin;
    if (origin) {
      const keys: readonly string[] = origin.consumedLazyKeys;
      void keys;
      // @ts-expect-error Nested receipt arrays are immutable.
      origin.consumedLazyKeys.push('key');
    }
    // @ts-expect-error Compilation origins have no public setter.
    stats.compilation.rebuildOrigin = undefined;
  });
};
void observeWatchOrigin;
