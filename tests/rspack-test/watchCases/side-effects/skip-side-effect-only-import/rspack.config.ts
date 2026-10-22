import { defineConfig, definePlugin } from '@rspack/cli';

let step = 0;

export default defineConfig({
  optimization: {
    sideEffects: true,
  },
  plugins: [
    definePlugin(function (compiler) {
      let identifiers: string[] = [];
      compiler.hooks.compilation.tap('TestPlugin', (compilation) => {
        identifiers = [];
        compilation.hooks.buildModule.tap('TestPlugin', (module) => {
          identifiers.push(module.identifier());
        });
      });
      compiler.hooks.done.tap('TestPlugin', () => {
        const fooBuilds = identifiers.filter((identifier) =>
          identifier.endsWith('foo.js'),
        ).length;

        if (step === 0) {
          expect(fooBuilds).toBe(0);
        } else if (step === 1) {
          expect(fooBuilds).toBe(0);
        } else if (step === 2) {
          expect(fooBuilds).toBe(1);
        } else {
          throw new Error('Unexpected step');
        }

        step += 1;
      });
    }),
  ],
});
