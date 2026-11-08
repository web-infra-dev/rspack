import path from 'node:path';
import { defineConfig } from '@rspack/cli';
import { ConcatenatedModule, NormalModule, type Compiler } from '@rspack/core';

export default (['webpack', 'rspack'] as const).flatMap((runtimeMode) =>
  [false, true].map((concatenateSharedEntry) =>
    defineConfig({
      mode: 'production',
      entry: {
        entry1: ['./index.js', './bar.js'],
        entry2: ['./index-alt.js', './bar.js'],
      },
      output: {
        filename: `${runtimeMode}-${concatenateSharedEntry}-[name].js`,
      },
      experiments: { runtimeMode },
      resolve: {
        alias: {
          './bar.js': path.resolve(
            import.meta.dirname,
            concatenateSharedEntry ? 'bar-concatenated.js' : 'bar.js',
          ),
        },
      },
      optimization: {
        concatenateModules: true,
        // Keep value.js available for the shared entry's concatenation group.
        inlineExports: !concatenateSharedEntry,
        minimize: false,
      },
      plugins: [
        {
          apply(compiler: Compiler) {
            compiler.hooks.afterCompile.tap(
              'CheckCopiedConnections',
              (compilation) => {
                const graph = compilation.moduleGraph;
                const modules = [...compilation.modules];
                const foo = modules.find(
                  (m) =>
                    m instanceof NormalModule &&
                    m.resource === path.join(import.meta.dirname, 'foo.js'),
                );
                expect(foo).toBeTruthy();
                const groups = modules.filter(
                  (m) =>
                    m instanceof ConcatenatedModule && m.modules.includes(foo!),
                );
                expect(groups.length).toBe(2);
                for (const original of graph.getOutgoingConnections(foo!)) {
                  const copies = [
                    ...graph.getIncomingConnections(original.module!),
                  ].filter((c) => c.dependency === original.dependency);
                  expect(copies.length).toBe(3);
                  expect(new Set(copies).size).toBe(3);
                  expect(new Set(copies.map((c) => c.originModule))).toEqual(
                    new Set([foo, ...groups]),
                  );
                  expect(graph.getConnection(original.dependency!)).toBe(
                    original,
                  );
                  for (const copy of copies) {
                    expect([
                      ...graph.getOutgoingConnections(copy.originModule!),
                    ]).toContain(copy);
                    expect(copy.dependency?._parentModule).toBe(foo);
                  }
                }
              },
            );
          },
        },
      ],
    }),
  ),
);
