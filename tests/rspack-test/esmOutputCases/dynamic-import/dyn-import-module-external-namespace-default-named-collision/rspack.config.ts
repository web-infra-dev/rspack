import { defineConfig, definePlugin } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  externals: {
    './foo-runtime.mjs': 'module ./foo-runtime.mjs',
  },
  plugins: [
    definePlugin({
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
    }),
  ],
});
