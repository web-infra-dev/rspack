module.exports = {
  plugins: [
    (compiler) => {
      compiler.hooks.compilation.tap('CheckNestedBlocks', (compilation) => {
        compilation.hooks.finishModules.tap('CheckNestedBlocks', (modules) => {
          const entry = [...modules].find((module) =>
            module.resource?.endsWith('/index.js'),
          );
          let blocks = entry.blocks;
          for (let depth = 0; depth < 4; depth++) {
            expect(blocks).toHaveLength(1);
            blocks = blocks[0].blocks;
          }
          expect(blocks).toHaveLength(0);
        });
      });
    },
  ],
};
