import { defineConfig, definePlugin } from '@rspack/cli';
import { NormalModuleReplacementPlugin } from '@rspack/core';
import path from 'node:path';

export default defineConfig({
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: /(?:instrumentation|hook)-stub\.js$/,
        loader: path.join(import.meta.dirname, 'loader.js'),
      },
    ],
  },
  plugins: [
    new NormalModuleReplacementPlugin(
      /instrumentation-stub\.js$/,
      ({ createData }) => {
        if (createData) {
          createData.context = import.meta.dirname;
        }
      },
    ),
    definePlugin({
      apply(compiler) {
        compiler.hooks.normalModuleFactory.tap('context-plugin', (factory) => {
          factory.hooks.afterResolve.tap(
            { name: 'context-plugin', stage: -1 },
            ({ createData }) => {
              if (createData?.resource.endsWith('hook-stub.js')) {
                createData.context = import.meta.dirname;
              }
            },
          );
          // The native replacement plugin at stage 0 separates these JS taps,
          // so the observer reads the context back from Rust.
          factory.hooks.afterResolve.tap(
            { name: 'context-observer', stage: 1 },
            ({ createData }) => {
              if (
                /(?:instrumentation|hook)-stub\.js$/.test(createData!.resource)
              ) {
                expect(createData!.context).toBe(import.meta.dirname);
              }
            },
          );
        });
      },
    }),
  ],
});
