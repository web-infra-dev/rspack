import { defineConfig, definePlugin } from '@rspack/cli';
import { rspack } from '@rspack/core';

let compilerIndex = 0;
const ENV = {
  ENV_A: 0,
  ENV_B: 0,
  ENV_C: 0,
};

export default defineConfig({
  entry: './entry.js',
  optimization: {
    minimize: false,
  },
  plugins: [
    definePlugin({
      apply(compiler) {
        const index = compilerIndex++;
        if (index === 1) {
          ENV.ENV_A++;
        }
        if (index === 2) {
          ENV.ENV_B++;
        }
        if (index === 3 || index === 6) {
          ENV.ENV_C++;
        }

        let definitions = {};
        if (index === 4) {
          definitions = { ADDED: JSON.stringify('added') };
        } else if (index === 5) {
          definitions = {
            ADDED: JSON.stringify('added'),
            NESTED: { value: JSON.stringify('nested') },
            'typeof OPTIONAL': JSON.stringify('number'),
          };
        } else if (index === 6) {
          // Reordering keys and changing unused ENV_C must preserve cache reuse.
          definitions = {
            'typeof OPTIONAL': JSON.stringify('number'),
            NESTED: { value: JSON.stringify('nested') },
            ADDED: JSON.stringify('added'),
          };
        }
        new rspack.DefinePlugin(definitions).apply(compiler);

        let builtModules = 0;
        compiler.hooks.compilation.tap('DefinePluginTest', (compilation) => {
          compilation.hooks.buildModule.tap('DefinePluginTest', () => {
            builtModules++;
          });
        });
        compiler.hooks.done.tap('DefinePluginTest', () => {
          expect(builtModules).toBe([4, 2, 2, 1, 4, 4, 0, 4, 0][index]);
        });
      },
    }),
    // This plugin must not hide changes to the earlier plugin's keys.
    new rspack.DefinePlugin({
      'process.env': ENV,
    }),
  ],
});
