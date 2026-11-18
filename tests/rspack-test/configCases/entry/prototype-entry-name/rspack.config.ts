import { defineConfig } from '@rspack/cli';
import { rspack, type Compiler } from '@rspack/core';

const variants: (string | string[] | { import: string; runtime: false })[] = [
  './index.js',
  ['./index.js'],
  { import: './index.js', runtime: false },
];

export default defineConfig({
  // A function keeps the special key intact through the test harness's config merge.
  entry: () => ({ ['__proto__']: './index.js' }),
  plugins: [
    (compiler: Compiler) => {
      compiler.hooks.beforeRun.tapPromise('CheckEntryName', async () => {
        for (const value of variants) {
          const entry = { ['__proto__']: value };
          for (const input of [entry, () => entry, async () => entry]) {
            const options = rspack.config.getNormalizedRspackOptions({
              entry: input,
            });
            const normalized =
              typeof options.entry === 'function'
                ? await options.entry()
                : options.entry;
            expect(
              Object.prototype.hasOwnProperty.call(normalized, '__proto__'),
            ).toBe(true);
            expect(Object.getPrototypeOf(normalized)).toBe(Object.prototype);
            expect(normalized['__proto__'].import).toEqual(['./index.js']);
          }
        }
      });
    },
  ],
});
