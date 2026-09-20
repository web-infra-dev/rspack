import path from 'node:path';

const kinds = ['file', 'context', 'missing', 'build'];

/** @type {import('@rspack/core').Configuration} */
export default {
  entry: './index.mjs',
  module: {
    rules: [{ test: /changing\.txt$/, use: ['./loader.mjs'] }],
  },
  plugins: [
    (compiler) => {
      let step = 0;
      const previous = new Map();
      compiler.hooks.thisCompilation.tap(
        'FileSystemDependencies',
        (compilation) => {
          const name = ['a', 'b', 'a'][step++];
          expect(name).toBeDefined();
          const prefix =
            path.join(compiler.options.context, 'tracked') + path.sep;
          const tracked = (values) =>
            Array.from(values)
              .filter((value) => value.startsWith(prefix))
              .sort();
          const collections = new Map();
          for (const kind of kinds) {
            const deps = compilation[`${kind}Dependencies`];
            collections.set(kind, deps);
            // Prime the snapshot before the loader changes native dependencies.
            if (kind !== 'file') Array.from(deps);
          }
          compilation.hooks.afterSeal.tapPromise(
            'FileSystemDependencies',
            async () => {
              for (const kind of kinds) {
                const deps = collections.get(kind);
                expect(compilation[`${kind}Dependencies`]).toBe(deps);
                const current = path.join(prefix, `${name}.${kind}`);
                const removed = path.join(
                  prefix,
                  `${name === 'a' ? 'b' : 'a'}.${kind}`,
                );
                const shared = path.join(prefix, `shared.${kind}`);
                const expected = [current, shared].sort();
                expect(tracked(deps)).toEqual(expected);
                expect(tracked(deps.values())).toEqual(expected);
                expect(tracked(deps.keys())).toEqual(expected);
                expect(deps.has(current)).toBe(true);
                expect(deps.has(shared)).toBe(true);
                expect(deps.has(removed)).toBe(false);
                expect(deps.size).toBe(new Set(deps).size);

                // The loader and a plugin can register the same native path.
                // After flush there is no pending JS set to hide duplicates.
                const size = deps.size;
                deps.addAll([shared, shared]);
                await Promise.resolve();
                expect(tracked(deps.values())).toEqual(expected);
                expect(deps.size).toBe(size);
                const pluginAdded = path.join(
                  compiler.context,
                  `plugin-${name}-依赖🦀.${kind}`,
                );
                deps.add(pluginAdded);
                await Promise.resolve();
                expect(deps.has(pluginAdded)).toBe(true);
                expect(Array.from(deps)).toContain(pluginAdded);

                // A previously returned iterator remains a snapshot of that build.
                const last = previous.get(kind);
                if (last) {
                  expect(tracked(last.iterator)).toEqual(last.expected);
                  // Retained wrappers read the current compilation after rebuild.
                  expect(last.compilation[`${kind}Dependencies`]).toBe(
                    last.deps,
                  );
                  expect(last.deps).not.toBe(deps);
                  expect(tracked(last.deps)).toEqual(expected);
                  expect(last.deps.has(current)).toBe(true);
                  expect(last.deps.has(removed)).toBe(false);
                  expect(last.deps.size).toBe(deps.size);

                  // Old wrappers add to the current compilation, but deletions
                  // remain local to the wrapper, matching the previous JS API.
                  const lateAddition = path.join(
                    compiler.context,
                    `late-${name}.${kind}`,
                  );
                  last.deps.add(lateAddition);
                  await Promise.resolve();
                  expect(deps.has(lateAddition)).toBe(true);
                  expect(last.deps.delete(current)).toBe(true);
                  expect(last.deps.has(current)).toBe(false);
                  expect(tracked(last.deps)).toEqual([shared]);
                  expect(last.deps.size).toBe(deps.size - 1);
                  expect(deps.has(current)).toBe(true);
                  last.deps.add(current);
                  await Promise.resolve();
                  expect(tracked(last.deps)).toEqual(expected);
                }
                previous.set(kind, {
                  compilation,
                  deps,
                  iterator: deps.values(),
                  expected,
                });
              }
            },
          );
        },
      );
    },
  ],
};
