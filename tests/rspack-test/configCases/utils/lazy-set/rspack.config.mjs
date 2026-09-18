import path from 'node:path';

/** @type {import("@rspack/core").Configuration} */
export default {
  plugins: [
    (compiler) => {
      compiler.hooks.thisCompilation.tap('AddedStrings', (compilation) => {
        compilation.hooks.afterSeal.tapPromise('AddedStrings', async () => {
          for (const kind of ['file', 'context', 'missing', 'build']) {
            const deps = compilation[`${kind}Dependencies`];
            const originalSize = deps.size;
            const first = path.join(compiler.context, `${kind}-依赖🦀.txt`);
            const second = path.join(compiler.context, `${kind}-second.txt`);
            const third = path.join(compiler.context, `${kind}-third.txt`);

            // Flush separate batches before the first read so their strings
            // must survive the N-API calls that supplied them.
            deps.add(first);
            await Promise.resolve();
            deps.addAll([first, second, second]);
            await Promise.resolve();
            expect(deps.has(first)).toBe(true);
            expect(deps.size).toBe(originalSize + 2);
            const snapshot = deps.values();
            expect(Array.from(deps)).toEqual(
              expect.arrayContaining([first, second]),
            );

            // Reuse cached strings while incorporating another batch. An
            // iterator already returned to JS must keep its previous snapshot.
            deps.addAll([first, third, third]);
            expect(deps.has(third)).toBe(true);
            expect(deps.size).toBe(originalSize + 3);
            expect(Array.from(deps).filter((value) => value === third)).toEqual(
              [third],
            );
            await Promise.resolve();
            const values = Array.from(deps);
            expect(values).toEqual(
              expect.arrayContaining([first, second, third]),
            );
            expect(values.length).toBe(originalSize + 3);
            expect(Array.from(snapshot)).not.toContain(third);
            expect(Array.from(deps)).toEqual(values);
          }
        });
      });
      compiler.hooks.done.tap('Test', ({ compilation }) => {
        const items1 = Array.from(compilation.fileDependencies);
        const items2 = new Set(compilation.fileDependencies.keys());
        const items3 = new Set(compilation.fileDependencies.values());
        const items4 = new Set(compilation.fileDependencies.entries());
        expect(compilation.fileDependencies.has(items1[0])).toBe(true);
        compilation.fileDependencies.delete(items1[0]);
        expect(compilation.fileDependencies.has(items1[0])).toBe(false);
        compilation.fileDependencies.add(items1[0]);
        expect(compilation.fileDependencies.has(items1[0])).toBe(true);
        compilation.fileDependencies.add(items1[0]);
        expect(compilation.fileDependencies.size).toBe(items1.length);
        const items1Set = new Set(items1);
        expect(items2).toEqual(items1Set);
        expect(items3).toEqual(items1Set);
        expect(items4).toEqual(new Set(items1.map((x) => [x, x])));
      });
    },
  ],
};
