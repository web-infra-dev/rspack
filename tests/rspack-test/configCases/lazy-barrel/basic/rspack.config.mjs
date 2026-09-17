import path from 'node:path';
import fs from 'node:fs';

const allModules = fs
  .readdirSync(import.meta.dirname, { recursive: true, withFileTypes: true })
  .filter(
    (dirent) =>
      dirent.isFile() &&
      dirent.name !== 'package.json' &&
      dirent.name !== 'rspack.config.mjs' &&
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

export default /** @type {import("@rspack/core").Configuration} */ ({
  experiments: {
    lazyBarrel: true,
  },
  plugins: [
    function (compiler) {
      const createdModules = new Set();
      compiler.hooks.thisCompilation.tap(
        'Test',
        (compilation, { normalModuleFactory }) => {
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
    },
  ],
});
