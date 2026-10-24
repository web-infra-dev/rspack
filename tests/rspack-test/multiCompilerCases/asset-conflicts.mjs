import path from "node:path";
import { createFsFromVolume, Volume } from "memfs";

const fixtureContext = path.join(import.meta.dirname, "../fixtures");

const options = (
  context,
  entries,
  outputPaths = entries.map(() => context.getDist())
) =>
  Object.assign(
    entries.map((entry, index) => ({
      name: String.fromCharCode(97 + index),
      context: fixtureContext,
      entry: `./${entry}.js`,
      output: {
        path: outputPaths[index],
        filename: "bundle.js"
      }
    })),
    { parallelism: 1 }
  );

const run = (compiler, assertion) =>
  new Promise((resolve, reject) => {
    compiler.run((error, stats) => {
      if (error) return reject(error);
      try {
        assertion(stats);
        compiler.close(resolve);
      } catch (error) {
        compiler.close(() => reject(error));
      }
    });
  });

const getWarnings = stats =>
  stats.stats.flatMap(child =>
    child.compilation.warnings.map(warning => warning.message)
  );

/** @type {import('@rspack/test-tools').TMultiCompilerCaseConfig[]} */
export default [
  {
    description: "should warn when compilers emit different content to the same file",
    options(context) {
      return options(context, ["a", "b"]);
    },
    build(context, compiler) {
      return run(compiler, stats => {
        expect(stats.hasWarnings()).toBe(true);
        const warnings = getWarnings(stats);
        expect(warnings).toHaveLength(1);
        expect(warnings[0]).toContain(
          'Conflict: Compilers "a" and "b" emit different content to the same filename bundle.js'
        );
      });
    }
  },
  {
    description: "should not warn when compilers emit identical content to the same file",
    options(context) {
      return options(context, ["a", "a"]);
    },
    build(context, compiler) {
      return run(compiler, stats => {
        expect(stats.hasWarnings()).toBe(false);
      });
    }
  },
  {
    description: "should not warn for the same filename in different output paths",
    options(context) {
      return options(context, ["a", "b"], [
        context.getDist("a"),
        context.getDist("b")
      ]);
    },
    build(context, compiler) {
      return run(compiler, stats => {
        expect(stats.hasWarnings()).toBe(false);
      });
    }
  },
  {
    description: "should not warn when compilers use different output file systems",
    options(context) {
      return options(context, ["a", "b"]);
    },
    compiler(context, compiler) {
      compiler.compilers[1].outputFileSystem = createFsFromVolume(new Volume());
    },
    build(context, compiler) {
      return run(compiler, stats => {
        expect(stats.hasWarnings()).toBe(false);
      });
    }
  },
  {
    description: "should retain other compiler outputs when checking watch rebuilds",
    options(context) {
      return options(context, ["a", "b"]);
    },
    build(context, compiler) {
      return new Promise((resolve, reject) => {
        let build = 0;
        compiler.watch({}, (error, stats) => {
          if (error) return reject(error);
          try {
            expect(getWarnings(stats)).toHaveLength(1);
            if (build++ === 0) {
              compiler.compilers[1].watching.invalidate();
            } else {
              compiler.close(resolve);
            }
          } catch (error) {
            compiler.close(() => reject(error));
          }
        });
      });
    }
  }
];
