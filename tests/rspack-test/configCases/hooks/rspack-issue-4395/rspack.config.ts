import { defineConfig, definePlugin } from '@rspack/cli';

const config = {
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.hooks.make.tapAsync('child', (base, callback) => {
          const child = base.createChildCompiler('child', {});
          child.runAsChild((error) => callback(error));
        });
      },
    }),
    definePlugin({
      apply(compiler) {
        let count = 0;
        const add = () => {
          count += 1;
        };
        const sub = () => {
          count -= 1;
        };

        compiler.hooks.thisCompilation.tap('mock-plugin', (compilation) => {
          compilation.hooks.processAssets.tap('mock-plugin', () => {
            sub();
            if (count === 0) {
              const { RawSource } = compiler.rspack.sources;
              compilation.emitAsset('./temp', new RawSource(''), {});
            }
          });
        });
        compiler.hooks.run.tap('mock-plugin', add);
      },
    }),
  ],
};

export default defineConfig(config);
