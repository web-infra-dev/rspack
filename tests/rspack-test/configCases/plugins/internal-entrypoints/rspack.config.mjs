import sources from 'webpack-sources';

const { ConcatSource } = sources;

/**
 * @type {import('@rspack/core').RspackOptions}
 */
export default {
  entry: {
    main: './index.js',
  },
  plugins: [
    {
      name: 'test',
      apply(compiler) {
        compiler.hooks.compilation.tap('compilation', (compilation) => {
          compilation.hooks.processAssets.tapPromise(
            'processAssets1',
            async (assets) => {
              let inspect = new ConcatSource();
              for (let [n, cg] of compilation.entrypoints) {
                inspect.add(`entry name: ${n}\n`);
                for (let file of cg.getFiles()) {
                  inspect.add(`  file: ${file}\n`);
                }
              }
              compilation.emitAsset('inspect.txt', inspect);
            },
          );
        });
      },
    },
  ],
};
