import rspack, {
  type ResolveData,
  type RspackOptions,
  workerFunction,
} from '@rspack/core';
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

const workerHook: (data: ResolveData) => Promise<false | void> = workerFunction(
  './rewrite.cjs',
  { from: 'virtual', to: './entry.js' },
);
const typedWorkerHook = rspack.workerFunction<typeof workerHook>(
  './rewrite.cjs',
  {},
);
const workerPlugin = definePlugin({
  apply(compiler) {
    compiler.hooks.normalModuleFactory.tap('worker-types', (factory) => {
      factory.hooks.beforeResolve.tapPromise('worker', typedWorkerHook);
      factory.hooks.beforeResolve.tapPromise(
        'inferred',
        workerFunction('./rewrite.cjs', {}),
      );
    });
  },
});
export const workerConfig = defineConfig({ plugins: [workerPlugin] });
