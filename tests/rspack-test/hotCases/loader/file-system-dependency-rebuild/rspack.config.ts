import path from 'node:path';
import { defineConfig } from '@rspack/cli';
import type { Compilation, Compiler } from '@rspack/core';

const kinds = ['file', 'context', 'missing', 'build'] as const;
type DependencyKind = (typeof kinds)[number];
type Dependencies = Compilation['fileDependencies'];

export default defineConfig({
  entry: './index.mjs',
  module: {
    rules: [{ test: /changing\.txt$/, use: ['./loader.mjs'] }],
  },
  plugins: [
    (compiler: Compiler) => {
      let step = 0;
      const previous = new Map<
        DependencyKind,
        { compilation: Compilation; dependencies: Dependencies }
      >();
      compiler.hooks.thisCompilation.tap(
        'FileSystemDependencyRebuild',
        (compilation: Compilation) => {
          const name = ['a', 'b', 'a'][step++];
          if (!name) throw new Error('Unexpected HMR step');
          const prefix =
            path.join(compiler.options.context!, 'tracked') + path.sep;
          const tracked = (values: Iterable<string>) =>
            Array.from(values)
              .filter((value) => value.startsWith(prefix))
              .sort();
          const collections = new Map<DependencyKind, Dependencies>();
          for (const kind of kinds) {
            const dependencies = compilation[`${kind}Dependencies`];
            collections.set(kind, dependencies);
            // Prime the cached array before the loader records dependencies.
            if (kind !== 'file') Array.from(dependencies);
          }
          compilation.hooks.afterSeal.tapPromise(
            'FileSystemDependencyRebuild',
            async () => {
              for (const kind of kinds) {
                const dependencies = collections.get(kind)!;
                expect(compilation[`${kind}Dependencies`]).toBe(dependencies);
                const current = path.join(prefix, `${name}.${kind}`);
                const removed = path.join(
                  prefix,
                  `${name === 'a' ? 'b' : 'a'}.${kind}`,
                );
                const shared = path.join(prefix, `shared.${kind}`);
                const expected = [current, shared].sort();
                expect(tracked(dependencies)).toEqual(expected);
                expect(dependencies.has(current)).toBe(true);
                expect(dependencies.has(shared)).toBe(true);
                expect(dependencies.has(removed)).toBe(false);

                const last = previous.get(kind);
                if (last) {
                  // Retained wrappers follow the current compilation while
                  // remaining cached on the compilation that exposed them.
                  expect(last.compilation[`${kind}Dependencies`]).toBe(
                    last.dependencies,
                  );
                  expect(last.dependencies).not.toBe(dependencies);
                  expect(tracked(last.dependencies)).toEqual(expected);

                  // Mutations through an old wrapper target the current
                  // compilation; deletions remain local to that wrapper.
                  const lateAddition = path.join(
                    compiler.context,
                    `late-${name}.${kind}`,
                  );
                  last.dependencies.add(lateAddition);
                  await Promise.resolve();
                  expect(dependencies.has(lateAddition)).toBe(true);
                  expect(last.dependencies.delete(current)).toBe(true);
                  expect(tracked(last.dependencies)).toEqual([shared]);
                  expect(dependencies.has(current)).toBe(true);
                  last.dependencies.add(current);
                  await Promise.resolve();
                  expect(tracked(last.dependencies)).toEqual(expected);
                }
                previous.set(kind, { compilation, dependencies });
              }
            },
          );
        },
      );
    },
  ],
});
