let compilerIndex = 0;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  context: __dirname,
  experiments: {
    newCache: {
      module: false,
    },
  },
  cache: {
    type: 'persistent',
  },
  plugins: [
    {
      apply(compiler) {
        let builtModules = 0;
        compiler.hooks.compilation.tap('Test Plugin', (compilation) => {
          compilation.hooks.buildModule.tap('Test Plugin', () => {
            builtModules++;
          });
        });
        compiler.hooks.done.tap('Test Plugin', () => {
          if (compilerIndex === 0) {
            expect(builtModules).toBeGreaterThan(0);
          } else {
            // The legacy make cache restores the whole module graph, so a warm
            // start must not build any module again.
            expect(builtModules).toBe(0);
          }
          compilerIndex++;
        });
      },
    },
  ],
};
