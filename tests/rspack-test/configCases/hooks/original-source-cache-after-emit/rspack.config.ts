import { defineConfig } from '@rspack/cli';
import type { Compiler, Module } from '@rspack/core';
import type { Source } from 'webpack-sources';

export default ([false, 'source-map'] as const).flatMap((devtool) =>
  [true, false].map((emit) =>
    defineConfig({
      devtool,
      optimization: { concatenateModules: false },
      plugins: [
        {
          apply(compiler: Compiler) {
            let module: Module;
            let retainedSource: Source;
            let cachedBinding: ReturnType<Module['_originalSource']>;
            let afterEmit = false;

            compiler.hooks.compilation.tap(
              'OriginalSourceCache',
              (compilation) => {
                compilation.hooks.processAssets.tap(
                  'OriginalSourceCache',
                  () => {
                    module = [...compilation.modules].find((module) =>
                      module.identifier().endsWith('index.js'),
                    )!;
                    expect(module).toBeDefined();
                    retainedSource = module.originalSource()!;
                    cachedBinding = module._originalSource();
                    expect(cachedBinding).toBeDefined();
                    expect(module._originalSource()).toBe(cachedBinding);
                    Object.freeze(module);
                  },
                );
              },
            );

            compiler.hooks.shouldEmit.tap('OriginalSourceCache', () => emit);
            compiler.hooks.afterEmit.tapPromise(
              { name: 'OriginalSourceCache', stage: Number.MAX_SAFE_INTEGER },
              async () => {
                await Promise.resolve();
                expect(module._originalSource()).toBe(cachedBinding);
                afterEmit = true;
              },
            );
            compiler.hooks.done.tap('OriginalSourceCache', () => {
              expect(afterEmit).toBe(emit);
              const binding = module._originalSource();
              expect(binding).toBeDefined();
              expect(binding).not.toBe(cachedBinding);
              expect(module._originalSource()).toBe(binding);

              // A Source retained by a plugin still owns its content and lazy map.
              const source = module.originalSource()!;
              expect(source.source()).toEqual(retainedSource.source());
              expect(source.map()).toEqual(retainedSource.map());
              if (devtool) {
                expect(retainedSource.map()?.sourcesContent?.[0]).toContain(
                  'original source cache',
                );
              }
            });
          },
        },
      ],
    }),
  ),
);
