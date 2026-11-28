import { execFile } from 'node:child_process';
import path from 'node:path';
import { promisify } from 'node:util';
import { defineConfig, definePlugin } from '@rspack/cli';

const run = promisify(execFile);
const pluginName = 'CacheRestoreChangeDetection';

export default defineConfig({
  experiments: { newCache: true },
  cache: { type: 'persistent' },
  plugins: [
    definePlugin({
      apply(compiler) {
        let buildCount = 0;
        compiler.hooks.watchRun.tapPromise(pluginName, async () => {
          if (buildCount !== 0) return;
          // The watch harness has copied 0/ and aged its file timestamps.
          // Warm in another process so its native paths cannot populate this
          // process's path interner before the first watch build restores them.
          await run(
            process.execPath,
            [
              path.join(import.meta.dirname, 'warm.mjs'),
              JSON.stringify({
                context: compiler.context,
                mode: compiler.options.mode,
                target: compiler.options.target,
                entry: compiler.options.entry,
                devtool: compiler.options.devtool,
                output: compiler.options.output,
                experiments: compiler.options.experiments,
                cache: compiler.options.cache,
              }),
            ],
            {
              cwd: compiler.context,
              windowsHide: true,
              timeout: 30_000,
            },
          );
        });
        compiler.hooks.done.tap(pluginName, (stats) => {
          const valueFile = path.join(compiler.context, 'value.js');
          expect([...stats.compilation.fileDependencies]).toContain(valueFile);
          buildCount++;
          if (buildCount === 1) {
            const modules = stats.toJson({ all: false, cacheInfo: true })
              .cacheInfo?.['|Compilation/modules'];
            expect(modules?.reused).toBe(2);
          } else {
            expect(buildCount).toBe(2);
            expect(compiler.modifiedFiles).toContain(valueFile);
          }
        });
      },
    }),
  ],
});
