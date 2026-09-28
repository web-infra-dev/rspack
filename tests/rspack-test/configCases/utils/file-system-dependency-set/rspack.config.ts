import path from 'node:path';
import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  plugins: [
    definePlugin((compiler) => {
      compiler.hooks.thisCompilation.tap(
        'FileSystemDependencySet',
        (compilation) => {
          compilation.hooks.afterSeal.tapPromise(
            'FileSystemDependencySet',
            async () => {
              const dependencies = compilation.fileDependencies;
              const initial = Array.from(dependencies);
              const first = path.join(compiler.context, 'first-依赖🦀.txt');
              const second = path.join(compiler.context, 'second.txt');
              const third = path.join(compiler.context, 'third.txt');

              // Values from separate batches stay valid, while duplicate
              // additions preserve Set semantics.
              dependencies.add(first);
              await Promise.resolve();
              dependencies.addAll([first, second, second]);
              await Promise.resolve();
              const expected = [...initial, first, second];
              expect(Array.from(dependencies)).toEqual(expected);
              expect(dependencies.size).toBe(new Set(expected).size);

              // Iterators use the native wrapper's shared array and can see a
              // later in-place refresh before iteration starts.
              const iterator = dependencies.values();
              dependencies.addAll([third, third]);
              await Promise.resolve();
              const updated = [...expected, third];
              expect(Array.from(dependencies)).toEqual(updated);
              expect(Array.from(iterator)).toEqual(updated);

              // Cancelling a pending deletion keeps the original order.
              expect(dependencies.delete(second)).toBe(true);
              dependencies.add(second);
              await Promise.resolve();
              expect(Array.from(dependencies)).toEqual(updated);

              // A delete can cancel an addition in the same batch.
              const cancelled = path.join(compiler.context, 'cancelled.txt');
              dependencies.add(cancelled);
              expect(dependencies.delete(cancelled)).toBe(true);
              expect(dependencies.delete(cancelled)).toBe(false);
              await Promise.resolve();
              expect(dependencies.has(cancelled)).toBe(false);

              // clear() includes additions that have not reached Rust yet.
              dependencies.add(cancelled);
              dependencies.clear();
              expect(Array.from(dependencies)).toEqual([]);
              await Promise.resolve();
              expect(dependencies.size).toBe(0);

              dependencies.addAll(updated);
              await Promise.resolve();
              expect(Array.from(dependencies)).toEqual(updated);
            },
          );
        },
      );
      compiler.hooks.done.tap('FileSystemDependencySet', ({ compilation }) => {
        const items = Array.from(compilation.fileDependencies);
        const itemSet = new Set(items);
        expect(new Set(compilation.fileDependencies.keys())).toEqual(itemSet);
        expect(new Set(compilation.fileDependencies.values())).toEqual(itemSet);
        expect(new Set(compilation.fileDependencies.entries())).toEqual(
          new Set(items.map((item) => [item, item])),
        );

        expect(compilation.fileDependencies.has(items[0])).toBe(true);
        compilation.fileDependencies.delete(items[0]);
        expect(compilation.fileDependencies.has(items[0])).toBe(false);
        compilation.fileDependencies.add(items[0]);
        compilation.fileDependencies.add(items[0]);
        expect(compilation.fileDependencies.size).toBe(items.length);
      });
    }),
  ],
});
