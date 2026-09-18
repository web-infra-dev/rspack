module.exports = {
  mode: 'development',
  plugins: [
    (compiler) => {
      let step = 0;
      compiler.hooks.compilation.tap('CheckNestedBlocks', (compilation) => {
        const nested = step++ !== 1;
        compilation.hooks.finishModules.tap('CheckNestedBlocks', (modules) => {
          const entry = [...modules].find(
            (module) => module.rawRequest === './index.js',
          );
          expect(entry.blocks).toHaveLength(1);
          expect(entry.blocks[0].blocks).toHaveLength(nested ? 1 : 0);
        });
      });
    },
  ],
};
