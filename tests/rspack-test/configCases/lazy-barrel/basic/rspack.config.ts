import path from 'node:path';
import fs from 'node:fs';
import { defineConfig, definePlugin } from '@rspack/cli';

const allModules = fs
  .readdirSync(import.meta.dirname, { recursive: true, withFileTypes: true })
  .filter(
    (dirent) =>
      dirent.isFile() &&
      dirent.name !== 'package.json' &&
      dirent.name !== 'rspack.config.ts' &&
      dirent.name !== 'test.filter.mjs',
  )
  .map((dirent) => path.resolve(dirent.parentPath ?? dirent.path, dirent.name));

const lazyModules = new Set(
  [
    'named-barrel/b.js',
    'mixed-barrel/a.js',
    'mixed-barrel/b.js',
    'star-barrel/c.js',
    'nested-barrel/c.js',
  ].map((filename) => path.resolve(import.meta.dirname, filename)),
);

export default defineConfig({
  plugins: [
    definePlugin((compiler) => {
      const createdModules = new Set<string>();
      compiler.hooks.thisCompilation.tap(
        'Test',
        (_compilation, { normalModuleFactory }) => {
          normalModuleFactory.hooks.createModule.tap('Test', (data) => {
            createdModules.add(data.resourceResolveData.resource);
          });
        },
      );
      compiler.hooks.done.tap('Test', () => {
        lazyModules.forEach((module) => {
          expect(createdModules.has(module)).toBe(false);
        });
        expect(
          allModules.filter(
            (module) => !createdModules.has(module) && !lazyModules.has(module),
          ).length,
        ).toBe(0);
      });
    }),
  ],
});
