import fs from 'node:fs';
import path from 'node:path';
import { defineConfig, definePlugin } from '@rspack/cli';
import type { PersistentCacheOptions } from '@rspack/core';

const storageDirectory = path.join(import.meta.dirname, 'cache');
let buildIndex = 0;

export default defineConfig({
  name: 'root',
  entry: './a.js',
  cache: {
    type: 'persistent',
    version: 'v1',
    storage: {
      type: 'filesystem',
      directory: storageDirectory,
    },
  },
  plugins: [
    definePlugin({
      apply(compiler) {
        if (compiler.isChild()) return;
        compiler.hooks.beforeCompile.tap('ChildCompilerStorage', () => {
          // Switching cache versions must not create another storage directory.
          const cache = compiler.options.cache as PersistentCacheOptions;
          cache.version = `v${buildIndex + 1}`;
        });
        compiler.hooks.make.tapAsync(
          'ChildCompilersPlugin',
          (compilation, callback) => {
            const runChildCompiler = (name: string) =>
              new Promise<void>((resolve, reject) => {
                const childCompiler = compilation.createChildCompiler(
                  name,
                  { filename: `${name}.js` },
                  [
                    new compiler.rspack.EntryPlugin(
                      compiler.context,
                      './a.js',
                      { name },
                    ),
                  ],
                );
                childCompiler.runAsChild((error) => {
                  childCompiler.close((closeError) => {
                    const finalError = error || closeError;
                    if (finalError) {
                      reject(finalError);
                    } else {
                      resolve();
                    }
                  });
                });
              });

            (async () => {
              for (const name of ['child-a', 'child-b']) {
                await runChildCompiler(name);
              }
            })().then(
              () => callback(),
              (error) => callback(error),
            );
          },
        );
        compiler.hooks.done.tap('ChildCompilerStorage', (stats) => {
          const cacheInfo =
            stats.toJson({ all: false, cacheInfo: true }).cacheInfo ?? {};
          expect(
            Object.keys(cacheInfo).some((key) => key.includes('|child')),
          ).toBe(true);
          if (buildIndex === 1) {
            // Parent and children share one active compiler cache directory.
            const entries = fs
              .readdirSync(storageDirectory)
              .filter(
                (name) =>
                  name !== 'loader' &&
                  name !== '_stale' &&
                  fs.statSync(path.join(storageDirectory, name)).isDirectory(),
              );
            expect(entries).toHaveLength(1);
          }
          buildIndex++;
        });
      },
    }),
  ],
});
