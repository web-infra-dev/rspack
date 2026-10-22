import path from 'node:path';
import fs from 'node:fs';
import { defineConfig } from '@rspack/cli';
import type { Compiler, Module, NormalModule } from '@rspack/core';

const PLUGIN_NAME = 'plugin';

class Plugin {
  apply(compiler: Compiler) {
    const { EntryPlugin, EntryDependency } = compiler.rspack;

    const modules: Record<string, Module> = {};

    const fooDependency = EntryPlugin.createDependency(
      path.resolve(import.meta.dirname, 'foo.js'),
    );
    const barDependency = EntryPlugin.createDependency(
      path.resolve(import.meta.dirname, 'bar.js'),
    );

    expect(fooDependency instanceof EntryDependency).toBeTruthy();

    compiler.hooks.finishMake.tapPromise(PLUGIN_NAME, async (compilation) => {
      const tasks = [];
      tasks.push(
        new Promise((resolve, reject) => {
          compilation.addInclude(
            compiler.context,
            fooDependency,
            {},
            (err, module) => {
              if (err) {
                reject(err);
                return;
              }
              const exportsInfo = compilation.moduleGraph.getExportsInfo(
                module!,
              );
              exportsInfo.setUsedInUnknownWay('main');
              modules['foo'] = module!;
              resolve(module);
            },
          );
        }),
      );
      tasks.push(
        new Promise((resolve, reject) => {
          compilation.addInclude(
            compiler.context,
            barDependency,
            {},
            (err, module) => {
              if (err) {
                reject(err);
                return;
              }
              const exportsInfo = compilation.moduleGraph.getExportsInfo(
                module!,
              );
              exportsInfo.setUsedInUnknownWay('main');
              modules['bar'] = module!;
              resolve(module);
            },
          );
        }),
      );
      tasks.push(
        new Promise((resolve) => {
          compilation.addInclude(
            compiler.context,
            EntryPlugin.createDependency(
              path.resolve(import.meta.dirname, 'no-exist.js'),
            ),
            {},
            (err, module) => {
              expect(err?.message).toMatch(/Can't resolve/);
              resolve(module);
            },
          );
        }),
      );
      await Promise.all(tasks);
    });

    const manifest: Record<string, string | number | null> = {};
    compiler.hooks.compilation.tap(PLUGIN_NAME, (compilation) => {
      compilation.hooks.processAssets.tap(PLUGIN_NAME, () => {
        for (const [key, module] of Object.entries(modules)) {
          const moduleId = compilation.chunkGraph.getModuleId(module);
          manifest[key] = moduleId;
        }
        if (!fs.existsSync(compiler.outputPath)) {
          fs.mkdirSync(compiler.outputPath, { recursive: true });
        }
        fs.writeFileSync(
          path.join(compiler.outputPath, 'manifest.json'),
          JSON.stringify(manifest),
          'utf-8',
        );

        const fooModule = compilation.moduleGraph.getModule(fooDependency);
        expect(path.normalize((fooModule as NormalModule).request)).toBe(
          path.resolve(import.meta.dirname, './foo.js'),
        );

        const barModule = compilation.moduleGraph.getModule(barDependency);
        expect(path.normalize((barModule as NormalModule).request)).toBe(
          path.resolve(import.meta.dirname, './bar.js'),
        );
      });
    });
  }
}

export default defineConfig({
  entry: './index.js',
  plugins: [new Plugin()],
});
