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

            // Re-adding pending deletions keeps the first addition order,
            // even when the deletions are cancelled in a different order.
            const ordered = ['a', 'b', 'c'].map((name) =>
              path.join(compiler.context, `${kind}-ordered-${name}.txt`),
            );
            deps.addAll([ordered[0], ordered[1], ordered[0], ordered[2]]);
            expect(deps.delete(ordered[0])).toBe(true);
            expect(deps.delete(ordered[1])).toBe(true);
            expect(deps.delete(ordered[0])).toBe(false);
            deps.addAll([ordered[1], ordered[0], ordered[1]]);
            await Promise.resolve();
            expect(Array.from(deps)).toEqual([...values, ...ordered]);
            for (const value of ordered) {
              expect(deps.delete(value)).toBe(true);
              expect(deps.delete(value)).toBe(false);
            }
            expect(Array.from(deps)).toEqual(values);

            // Native overlaps, repeated pending additions and deletion markers
            // must agree between size and every snapshot-producing operation.
            const fourth = path.join(compiler.context, `${kind}-fourth.txt`);
            const fifth = path.join(compiler.context, `${kind}-fifth.txt`);
            deps.addAll([first, fourth, fourth, fifth]);
            deps.delete(second);
            deps.delete(fifth);
            const expected = values.filter((value) => value !== second);
            expected.push(fourth);
            expect(deps.size).toBe(expected.length);
            expect(Array.from(deps)).toEqual(expected);
            const keys = deps.keys();
            const entries = deps.entries();
            const pendingSnapshot = deps.values();
            const lazyIterator = deps[Symbol.iterator]();

            deps.delete(fourth);
            deps.add(second);
            await Promise.resolve();
            expect(deps.size).toBe(values.length);
            expect(deps.has(fifth)).toBe(false);
            expect(Array.from(keys)).toEqual(expected);
            expect(Array.from(entries)).toEqual(
              expected.map((value) => [value, value]),
            );
            expect(Array.from(pendingSnapshot)).toEqual(expected);
            // Symbol.iterator keeps its existing lazy start boundary.
            expect(Array.from(lazyIterator)).toEqual(values);

            const visited = [];
            deps.forEach((value, key, collection) => {
              expect(key).toBe(value);
              expect(collection).toBe(deps);
              if (visited.length === 0) {
                deps.delete(third);
                deps.add(fourth);
                // A nested read replaces the cached snapshot, but this forEach
                // must finish iterating the snapshot it started with.
                const changed = Array.from(deps);
                expect(changed).not.toContain(third);
                expect(changed).toContain(fourth);
                expect(Array.from(deps)).toEqual(changed);
              }
              visited.push(value);
            });
            expect(visited).toEqual(values);

            // Mixed operations retain their order across a batch and across
            // an explicit read followed by the already scheduled microtask.
            const queued = path.join(compiler.context, `${kind}-queued.txt`);
            const absent = path.join(compiler.context, `${kind}-absent.txt`);
            expect(deps.delete(absent)).toBe(false);
            expect(deps.add(queued)).toBe(deps);
            expect(deps.delete(queued)).toBe(true);
            expect(deps.delete(queued)).toBe(false);
            deps.add(queued);
            expect(deps.size).toBe(new Set(deps).size);
            expect(deps.has(queued)).toBe(true);
            expect(deps.delete(queued)).toBe(true);
            await Promise.resolve();
            expect(deps.has(queued)).toBe(false);
            deps.add(queued);
            expect(Array.from(deps)).toContain(queued);
            expect(deps.delete(queued)).toBe(true);
            expect(Array.from(deps)).not.toContain(queued);
            deps.addAll([queued, queued]);
            // clear must include additions not yet submitted to Rust.
            // Clear also hides pending values after the queued native flush.
            deps.clear();
            expect(deps.size).toBe(0);
            expect(Array.from(deps)).toEqual([]);
            await Promise.resolve();
            expect(deps.size).toBe(0);
            expect(deps.delete(queued)).toBe(false);
            deps.addAll(values);
            await Promise.resolve();
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
