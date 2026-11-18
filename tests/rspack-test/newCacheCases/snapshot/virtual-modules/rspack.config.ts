import { defineConfig, definePlugin } from '@rspack/cli';
import { ContextModule, experiments } from '@rspack/core';
import { escapeSep } from '@rspack/test-tools/helper/win';
import fs from 'node:fs/promises';
import path from 'node:path';

const pluginName = 'VirtualModulesCacheTest';
const packagePath = 'node_modules/virtual-cache-package';
const unmanagedPath = `${packagePath}/unmanaged`;
const timestamp = 946684800000;
let compilerIndex = 0;

export default defineConfig({
  // Exercise the default timestamp-only module, resolve and context snapshots.
  mode: 'development',
  snapshot: {
    immutablePaths: [/[\\/]immutable(?:[\\/]|$)/],
    unmanagedPaths: [/[\\/]virtual-cache-package[\\/]unmanaged(?:[\\/]|$)/],
  },
  module: { rules: [{ test: /\.txt$/, type: 'asset/source' }] },
  ignoreWarnings: [
    /Can't resolve 'virtual-cache-package\/unmanaged\/added\.js'/,
  ],
  plugins: [
    definePlugin({
      apply(compiler) {
        const generation = compilerIndex++;
        const content = `module.exports = ${JSON.stringify(generation < 2 ? 'before' : 'after')};`;
        const modules: Record<string, string> = {
          'value.js': content,
          'immutable/value.js': content,
          [`${packagePath}/package.json`]: JSON.stringify({
            name: 'virtual-cache-package',
            version: generation < 4 ? '1.0.0' : '2.0.0',
          }),
          [`${packagePath}/value.js`]: content,
          [`${unmanagedPath}/value.js`]: content,
        };
        if (generation === 4 || generation === 5) {
          modules[`${unmanagedPath}/added.js`] = 'module.exports = "added";';
        }
        if (generation === 2 || generation === 3) {
          // Match the real file's timestamp encoding to catch untagged hashes.
          modules['transition/value.txt'] = `f${timestamp}`;
        }
        new experiments.VirtualModulesPlugin(modules).apply(compiler);

        compiler.hooks.beforeRun.tapPromise(pluginName, async () => {
          const writeRealFile = async (file: string, contents: string) => {
            const target = path.join(compiler.context, file);
            await fs.mkdir(path.dirname(target), { recursive: true });
            await fs.writeFile(target, contents);
            await fs.utimes(target, new Date(timestamp), new Date(timestamp));
          };
          if (generation === 0) {
            await writeRealFile(
              `${unmanagedPath}/real.js`,
              'module.exports = "real";',
            );
          }
          if (generation === 0 || generation === 4) {
            await writeRealFile('transition/value.txt', 'real');
          } else if (generation === 2) {
            await fs.unlink(
              path.join(compiler.context, 'transition/value.txt'),
            );
          }
        });

        const built: string[] = [];
        compiler.hooks.compilation.tap(pluginName, (compilation) => {
          compilation.hooks.buildModule.tap(pluginName, (module) => {
            if (module instanceof ContextModule) {
              built.push(
                escapeSep(
                  path.relative(
                    compiler.context,
                    module.identifier().split('|')[0],
                  ),
                ),
              );
            }
          });
        });
        compiler.hooks.done.tap(pluginName, () => {
          // Check invalidation separately from the generated context's contents.
          const changedContexts: Record<number, string[]> = {
            0: ['immutable', packagePath, unmanagedPath, 'transition'],
            2: [unmanagedPath, 'transition'],
            4: [packagePath, unmanagedPath, 'transition'],
            6: [unmanagedPath],
          };
          expect(built.sort()).toEqual(
            (changedContexts[generation] ?? []).sort(),
          );
        });
      },
    }),
  ],
});
