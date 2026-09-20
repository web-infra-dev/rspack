import path from "node:path";
import assert from "node:assert/strict";
import { rspack } from "@rspack/core";
import {
  runCompiler,
  closeCompiler,
} from "@rspack/test-tools/helper/lifecycle";

const kinds = ["file", "context", "missing", "build"];

export default {
  description:
    "shares dependency conversion across APIs and releases old wrapper caches between builds",
  options: () => ({
    mode: "development",
    entry: "./entry.mjs",
    module: { rules: [{ test: /entry\.mjs$/, use: ["./loader.mjs"] }] },
  }),
  async build(_context, compiler) {
    let generation = 0;
    const retained = [];
    const shared = path.join(compiler.context, "共享依赖🦀");
    compiler.hooks.thisCompilation.tap("DependencyStrings", (compilation) => {
      const current = path.join(compiler.context, `generation-${generation}`);
      compilation.hooks.afterSeal.tap("DependencyStrings", () => {
        const modules = compilation.modules;
        const entry = [...modules].find((module) =>
          module.resource?.endsWith("entry.mjs"),
        );
        assert(entry);
        for (const kind of kinds) {
          const info = entry.buildInfo[`${kind}Dependencies`];
          assert(info instanceof Set);
          assert(info.has(shared));
          const dependencies = compilation[`${kind}Dependencies`];
          dependencies.addAll([shared, current, current]);
          const values = [...dependencies];
          assert.equal(values.filter((value) => value === shared).length, 1);
          assert.equal(values.filter((value) => value === current).length, 1);
          assert.equal(dependencies.size, values.length);
          // Exercise partial writes, position moves, truncation and empty fills
          // through the public collection. Existing iterators stay snapshots.
          const snapshot = dependencies.values();
          const extra = path.join(current, `${kind}-extra`);
          assert(dependencies.delete(shared));
          dependencies.add(extra);
          const changed = values.filter((value) => value !== shared).concat(extra);
          assert.deepEqual([...dependencies], changed);
          assert.deepEqual([...dependencies], changed);
          assert(dependencies.delete(extra));
          assert.deepEqual([...dependencies], changed.slice(0, -1));
          dependencies.clear();
          assert.deepEqual([...dependencies], []);
          assert.equal(dependencies.size, 0);
          dependencies.addAll(values);
          assert.deepEqual([...dependencies], values);
          assert.deepEqual([...snapshot], values);
        }
        if (generation === 0) {
          // Cross the 16-bit boundary in both pool indices and result positions.
          const dependencies = compilation.fileDependencies;
          const original = [...dependencies];
          const paths = Array.from({ length: 65_540 }, (_, index) =>
            path.join(current, `wide-index-${index}`),
          );
          dependencies.addAll(paths);
          assert.deepEqual([...dependencies], original.concat(paths));
          assert(dependencies.delete(paths[65_536]));
          assert.deepEqual(
            [...dependencies],
            original.concat(paths.filter((_, index) => index !== 65_536)),
          );
          dependencies.clear();
          assert.deepEqual([...dependencies], []);
          dependencies.addAll(original);
          assert.deepEqual([...dependencies], original);
        }
        for (const old of retained) {
          assert.deepEqual([...old.snapshot], old.values);
          old.snapshot = old.dependencies.values();
          old.values = [...old.dependencies];
          assert(old.dependencies.has(current));
          assert(old.dependencies.has(shared));
        }
        const dependencies = compilation.fileDependencies;
        retained.push({
          dependencies,
          snapshot: dependencies.values(),
          values: [...dependencies],
        });
        if (generation === 2)
          throw new Error("dependency string cleanup failure");
      });
    });
    try {
      for (; generation < 5; generation++) {
        if (generation === 2) {
          await assert.rejects(runCompiler(compiler), (error) => {
            assert.match(error.message, /dependency string cleanup failure/);
            assert.equal(typeof error.code, "string");
            return true;
          });
        } else {
          const stats = await runCompiler(compiler);
          assert.equal(stats.hasErrors(), false);
        }
      }
      const foreignPath = path.join(compiler.context, "other-compiler");
      const other = rspack({
        mode: "development",
        context: compiler.context,
        entry: "./entry.mjs",
        output: { path: path.join(compiler.outputPath, "other") },
        plugins: [
          (other) =>
            other.hooks.thisCompilation.tap("OtherCompiler", (compilation) => {
              compilation.fileDependencies.add(foreignPath);
            }),
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
      // Closing must not change snapshots already handed to the caller.
      const snapshots = retained.map(({ dependencies }) => [...dependencies]);
      const iterators = retained.map(({ dependencies }) =>
        dependencies.values(),
      );
      const shutdownError = new Error("dependency shutdown failure");
      let failed = false;
      compiler.hooks.shutdown.tap("DependencyStrings", () => {
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
        assert.deepEqual([...iterator], snapshots[index]),
      );
      // Closing disables pooling and releases the JS materializer. Retained
      // wrappers still read the live compiler through the direct fallback.
      retained.forEach(({ dependencies }, index) =>
        assert.deepEqual([...dependencies], snapshots[index]),
      );
    } finally {
      await closeCompiler(compiler);
    }
  },
};
