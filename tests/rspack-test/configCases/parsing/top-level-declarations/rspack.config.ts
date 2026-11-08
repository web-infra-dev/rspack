import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  target: 'web',
  output: {
    filename: '[name].js',
  },
  module: {
    parser: {
      javascript: {
        createRequire: true,
        worker: ['*audioContext.audioWorklet.addModule()'],
      },
    },
  },
  plugins: [
    definePlugin(function testPlugin(compiler) {
      compiler.hooks.finishMake.tap('test', (compilation) => {
        for (const module of compilation.modules) {
          const name = module.nameForCondition();
          const topLevelDeclarations =
            module.buildInfo && module.buildInfo.topLevelDeclarations;
          if (
            name &&
            name.includes('top-level-declarations/index.js') &&
            topLevelDeclarations
          ) {
            const expectedTopLevelDeclarations = new Set([
              'a',
              'createRequire',
              'myRequire',
              'b',
              'c',
              'audioContext',
              'd',
            ]);
            expect(topLevelDeclarations).toEqual(expectedTopLevelDeclarations);
          }
        }
      });
    }),
  ],
});
