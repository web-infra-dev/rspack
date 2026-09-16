/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    bundlerInfo: {
      force: true,
    },
  },
  plugins: [
    (compiler) => {
      compiler.hooks.compilation.tap('test', (compilation) => {
        compilation.hooks.additionalTreeRuntimeRequirements.tap(
          'test',
          (_, set) => {},
        );
      });
    },
  ],
};
