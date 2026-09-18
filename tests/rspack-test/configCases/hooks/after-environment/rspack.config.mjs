/** @type {import("@rspack/core").Configuration} */
export default {
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.afterEnvironment.tap('getResolver', () => {
          expect(compiler.resolverFactory).toBeTruthy();
        });
      },
    },
  ],
};
