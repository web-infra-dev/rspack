import path from 'node:path';
import assert from 'node:assert/strict';
import { rspack } from '@rspack/core';
import { runNodeCase } from '@rspack/test-tools/helper/node-case';
import {
  runCompiler,
  closeCompiler,
} from '@rspack/test-tools/helper/lifecycle';

const kinds = ['file', 'context', 'missing', 'build'];

export default [
  {
    description: 'ties dependency array lifetime to its native wrapper',
    async run() {
      await runNodeCase(new URL('./dependency-array-gc.mjs', import.meta.url));
    },
  },
  {
    description:
      'reuses dependency strings and refreshes retained wrappers between builds',
    options: () => ({
      mode: 'development',
      entry: './entry.mjs',
      module: {
        rules: [{ test: /entry\.mjs$/, use: ['./dependency-loader.mjs'] }],
      },
    }),
    async build(_context, compiler) {
      let generation = 0;
      const retained = [];
      const shared = path.join(compiler.context, '共享依赖🦀');
      compiler.hooks.thisCompilation.tap(
        'FileSystemDependencyLifecycle',
        (compilation) => {
          const current = path.join(
            compiler.context,
            `generation-${generation}`,
          );
          compilation.hooks.afterSeal.tap(
            'FileSystemDependencyLifecycle',
            () => {
              const modules = compilation.modules;
              const entry = [...modules].find((module) =>
                module.resource?.endsWith('entry.mjs'),
              );
              assert(entry);
              for (const kind of kinds) {
                const info = entry.buildInfo[`${kind}Dependencies`];
                assert(info instanceof Set);
                assert(info.has(shared));
                const dependencies = compilation[`${kind}Dependencies`];
                dependencies.addAll([shared, current, current]);
                const values = [...dependencies];
                assert.equal(
                  values.filter((value) => value === shared).length,
                  1,
                );
                assert.equal(
                  values.filter((value) => value === current).length,
                  1,
                );
                assert.equal(dependencies.size, values.length);
              }

              // One collection is enough to exercise partial writes, position
              // moves, truncation and empty fills in the shared update protocol.
              const fileDependencies = compilation.fileDependencies;
              const original = [...fileDependencies];
              const extra = path.join(current, 'extra');
              assert(fileDependencies.delete(shared));
              fileDependencies.add(extra);
              const changed = original
                .filter((value) => value !== shared)
                .concat(extra);
              assert.deepEqual([...fileDependencies], changed);
              assert(fileDependencies.delete(extra));
              assert.deepEqual(
                [...fileDependencies],
                changed.slice(0, -1),
              );
              fileDependencies.clear();
              assert.deepEqual([...fileDependencies], []);
              fileDependencies.addAll(original);
              assert.deepEqual([...fileDependencies], original);

              if (generation === 0) {
                // Cross the 16-bit boundary in both pool indices and result positions.
                const paths = Array.from({ length: 65_540 }, (_, index) =>
                  path.join(current, `wide-index-${index}`),
                );
                fileDependencies.addAll(paths);
                assert.deepEqual(
                  [...fileDependencies],
                  original.concat(paths),
                );
                assert(fileDependencies.delete(paths[65_536]));
                assert.deepEqual(
                  [...fileDependencies],
                  original.concat(
                    paths.filter((_, index) => index !== 65_536),
                  ),
                );
                fileDependencies.clear();
                assert.deepEqual([...fileDependencies], []);
                fileDependencies.addAll(original);
                assert.deepEqual([...fileDependencies], original);
              }
              for (const old of retained) {
                assert([...old.dependencies].includes(current));
                assert(old.dependencies.has(current));
                assert(old.dependencies.has(shared));
              }
              retained.push({ dependencies: fileDependencies });
              if (generation === 2)
                throw new Error('dependency string cleanup failure');
            },
          );
        },
      );
      try {
        for (; generation < 5; generation++) {
          if (generation === 2) {
            await assert.rejects(runCompiler(compiler), (error) => {
              assert.match(error.message, /dependency string cleanup failure/);
              assert.equal(typeof error.code, 'string');
              return true;
            });
          } else {
            const stats = await runCompiler(compiler);
            assert.equal(stats.hasErrors(), false);
          }
        }
        const foreignPath = path.join(compiler.context, 'other-compiler');
        const other = rspack({
          mode: 'development',
          context: compiler.context,
          entry: './entry.mjs',
          output: { path: path.join(compiler.outputPath, 'other') },
          plugins: [
            (other) =>
              other.hooks.thisCompilation.tap(
                'OtherCompiler',
                (compilation) => {
                  compilation.fileDependencies.add(foreignPath);
                },
              ),
          ],
        });
        other.outputFileSystem = compiler.outputFileSystem;
        try {
          const stats = await runCompiler(other);
          assert(stats.compilation.fileDependencies.has(foreignPath));
          for (const { dependencies } of retained)
            assert(!dependencies.has(foreignPath));
        } finally {
          await closeCompiler(other);
        }
        // Closing must not invalidate arrays held by retained wrappers.
        const expectedValues = retained.map(({ dependencies }) => [
          ...dependencies,
        ]);
        const iterators = retained.map(({ dependencies }) =>
          dependencies.values(),
        );
        const shutdownError = new Error('dependency shutdown failure');
        let failed = false;
        compiler.hooks.shutdown.tap('FileSystemDependencyLifecycle', () => {
          if (!failed) {
            failed = true;
            throw shutdownError;
          }
        });
        await assert.rejects(
          closeCompiler(compiler),
          (error) => error === shutdownError,
        );
        iterators.forEach((iterator, index) =>
          assert.deepEqual([...iterator], expectedValues[index]),
        );
        // Closing preserves the JS-owned pool and JS helpers. Retained
        // wrappers keep using the same conversion path while the compiler lives.
        retained.forEach(({ dependencies }, index) =>
          assert.deepEqual([...dependencies], expectedValues[index]),
        );
      } finally {
        await closeCompiler(compiler);
      }
    },
  },
];
