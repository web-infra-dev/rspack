import { defineConfig } from '@rspack/cli';
import TestPlugin from '../stage-compilation/plugin.ts';

export default defineConfig({
  externals: ['commonjs fs', 'commonjs path'],
  plugins: [
    new TestPlugin((compiler, list) => {
      compiler.hooks.compilation.tap(
        TestPlugin.name,
        (_compilation, { normalModuleFactory }) => {
          normalModuleFactory.hooks.factorize.tap(
            TestPlugin.name,
            (resolveData) => {
              list.push(`/*: ${resolveData.request} :*/`);
            },
          );
        },
      );
    }),
  ],
});
