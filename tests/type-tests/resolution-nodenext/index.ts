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

import {
  Module,
  NormalModule,
  ConcatenatedModule,
  ContextModule,
  ExternalModule,
} from '@rspack/core';

export function moduleInheritance(
  normal: NormalModule,
  concatenated: ConcatenatedModule,
  context: ContextModule,
  external: ExternalModule,
): Module[] {
  class CustomModule extends NormalModule {
    override size(type?: string | null): number {
      return super.size(type);
    }
  }
  const constructor: typeof Module = CustomModule;
  const hooks = NormalModule.getCompilationHooks;
  void constructor;
  void hooks;
  return [normal, concatenated, context, external];
}
