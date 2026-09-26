import path from 'node:path';
import { defineConfig } from '@rspack/cli';
import type { Compiler, Module } from '@rspack/core';

const PLUGIN_NAME = 'Plugin';

class Plugin {
  apply(compiler: Compiler) {
    const { EntryPlugin } = compiler.rspack;

    const fooDependency = EntryPlugin.createDependency(
      path.resolve(import.meta.dirname, 'foo.js'),
    );
    const barDependency = EntryPlugin.createDependency(
      path.resolve(import.meta.dirname, 'bar.js'),
    );

    const modules: Record<string, Module | undefined> = {};

    compiler.hooks.make.tapPromise(PLUGIN_NAME, async (compilation) => {
      const tasks = [];
      tasks.push(
        new Promise<void>((resolve, reject) => {
          compilation.addEntry(
            compiler.context,
            fooDependency,
            'foo',
            (err, module) => {
              if (err) {
                reject(err);
                return;
              }
              modules.foo = module;
              resolve();
            },
          );
        }),
      );
      tasks.push(
        new Promise<void>((resolve, reject) => {
          compilation.addEntry(
            compiler.context,
            barDependency,
            {
              name: 'bar',
            },
            (err, module) => {
              if (err) {
                reject(err);
                return;
              }
              modules.bar = module;
              resolve();
            },
          );
        }),
      );
      await Promise.all(tasks);
    });

    compiler.hooks.compilation.tap(PLUGIN_NAME, (compilation) => {
      compilation.hooks.processAssets.tap(PLUGIN_NAME, () => {
        const fooModule = compilation.moduleGraph.getModule(fooDependency);
        expect(fooModule).toBe(modules.foo);
        expect(compilation.moduleGraph.getResolvedModule(fooDependency)).toBe(
          fooModule,
        );
        expect(
          compilation.moduleGraph.getConnection(fooDependency)?.module,
        ).toBe(fooModule);
        expect(
          compilation.moduleGraph.getParentModule(fooDependency),
        ).toBeNull();
        expect(compilation.moduleGraph.getParentBlockIndex(fooDependency)).toBe(
          -1,
        );

        const barModule = compilation.moduleGraph.getModule(barDependency);
        expect(barModule).toBe(modules.bar);
      });
    });
  }
}

export default defineConfig({
  output: {
    filename: '[name].js',
  },
  plugins: [new Plugin()],
});
