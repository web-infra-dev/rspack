/** @type {import("@rspack/core").Configuration} */
export default {
  optimization: {
    splitChunks: {
      minSize: 1,
    },
  },
  plugins: [
    (compiler) => {
      compiler.hooks.compilation.tap('CheckNestedBlocks', (compilation) => {
        compilation.hooks.finishModules.tap('CheckNestedBlocks', (modules) => {
          const entry = [...modules].find(
            (module) => module.rawRequest === './index.js',
          );
          expect(entry.blocks).toHaveLength(1);
          expect(entry.blocks[0].blocks).toHaveLength(3);
        });
      });
    },
  ],
};
