const path = require('path');

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
module.exports = {
  description:
    'should retain compilation artifacts after a rejected finishModules tap',
  options() {
    return {
      entry: './esm/a.js',
      optimization: { providedExports: true, usedExports: false },
    };
  },
  async build(context, compiler) {
    let calls = 0;
    let checkArtifacts;
    compiler.hooks.compilation.tap('Test', (compilation) => {
      compilation.hooks.finishModules.tapPromise(
        { name: 'Test', stage: 100 },
        async (modules) => {
          const module = [...modules].find(
            (module) =>
              module.resource === path.join(context.getSource(), 'esm/a.js'),
          );
          const exportsInfo = compilation.moduleGraph.getExportsInfo(module);
          checkArtifacts = () => {
            expect(
              compilation.moduleGraph.getProvidedExports(module).sort(),
            ).toEqual(['a', 'default']);
            expect(compilation.moduleGraph.isAsync(module)).toBe(false);
            expect(exportsInfo.isUsed('main')).toBe(true);
          };
          checkArtifacts();
          await new Promise((resolve) => setImmediate(resolve));
          checkArtifacts();
          if (++calls % 2 === 1) {
            throw new Error('finishModules failure');
          }
        },
      );
    });

    const run = () =>
      new Promise((resolve, reject) => {
        compiler.run((error, stats) => {
          if (error) reject(error);
          else resolve(stats);
        });
      });

    // Exercise error cleanup for both the initial build and a later rebuild.
    for (let attempt = 0; attempt < 2; attempt++) {
      await expect(run()).rejects.toThrow('finishModules failure');
      checkArtifacts();
      const stats = await run();
      expect(stats.hasErrors()).toBe(false);
    }
    expect(calls).toBe(4);
  },
};
