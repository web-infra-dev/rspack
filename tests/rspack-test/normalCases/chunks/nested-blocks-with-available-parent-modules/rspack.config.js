module.exports = {
  plugins: [
    (compiler) => {
      compiler.hooks.compilation.tap('CheckNestedBlocks', (compilation) => {
        compilation.hooks.finishModules.tap('CheckNestedBlocks', (modules) => {
          const entry = [...modules].find((module) =>
            module.resource?.replace(/\\/g, '/').endsWith('/index.js'),
          );
          expect(entry.blocks).toHaveLength(1);
          expect(entry.blocks[0].blocks).toHaveLength(1);
          expect(entry.blocks[0].blocks[0].blocks).toHaveLength(0);
        });
      });
    },
  ],
};
