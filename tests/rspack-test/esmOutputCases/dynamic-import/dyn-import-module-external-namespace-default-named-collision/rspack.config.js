const { rspack } = require('@rspack/core');

/** @type {import('@rspack/core').Configuration} */
module.exports = {
  externals: {
    './foo-runtime.mjs': 'module ./foo-runtime.mjs',
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.compilation.tap('testcase', (compilation) => {
          compilation.hooks.processAssets.tap(
            {
              name: 'testcase',
              stage:
                compiler.rspack.Compilation.PROCESS_ASSETS_STAGE_ADDITIONAL,
            },
            () => {
              compilation.emitAsset(
                'foo-runtime.mjs',
                new rspack.sources.RawSource(
                  [
                    "export const foo = 'named-foo';",
                    "export default 'default-foo';",
                    '',
                  ].join('\n'),
                ),
              );
            },
          );
        });
      },
    },
  ],
};
