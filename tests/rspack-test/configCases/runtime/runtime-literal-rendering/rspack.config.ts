import { defineConfig } from '@rspack/cli';

const publicPath = '/assets/\\\r\nnested/';
const version = String.raw`release\nbuild`;

export default defineConfig({
  mode: 'development',
  output: {
    filename: 'bundle0.js',
    publicPath,
    bundlerInfo: { version },
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.thisCompilation.tap(
          'RuntimeLiteralRendering',
          (compilation) => {
            compilation.hooks.processAssets.tap(
              'RuntimeLiteralRendering',
              () => {
                const source = compilation
                  .getAsset('bundle0.js')!
                  .source.source()
                  .toString();
                expect(source).toContain(`"${publicPath}"`);
                expect(source).toContain(`"${version}"`);
              },
            );
          },
        );
      },
    },
  ],
});
