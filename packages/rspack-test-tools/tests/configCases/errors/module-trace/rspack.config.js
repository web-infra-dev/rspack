/** @type {import("@rspack/core").Configuration} */
module.exports = {
  entry: "./index",
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.done.tap("TestPlugin", stats => {
          const errors = stats.toJson({ errors: true, ids: true }).errors;
          expect(errors.length).toBe(1);
          const moduleTrace = errors[0].moduleTrace;
          expect(moduleTrace[0].module.name).toBe('./c.js');
          expect(moduleTrace[0].origin.name).toBe('./b.js');
          expect(moduleTrace[1].module.name).toBe('./b.js');
          expect(moduleTrace[1].origin.name).toBe('./a.js');
          expect(moduleTrace[2].module.name).toBe('./a.js');
          expect(moduleTrace[2].origin.name).toBe('./index.js');
        });
      }
    }
  ]
};
