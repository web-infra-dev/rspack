import { defineConfig } from '@rspack/cli';
import { Compilation, type Compiler } from '@rspack/core';

class Plugin {
  apply(compiler: Compiler) {
    compiler.hooks.thisCompilation.tap('MyPlugin', (compilation) => {
      compilation.hooks.processAssets.tap(
        {
          name: 'MyPlugin',
          stage: Compilation.PROCESS_ASSETS_STAGE_OPTIMIZE_HASH,
        },
        () => {
          const cssFilename = compilation
            .getAssets()
            .find((asset) => asset.name.endsWith('.css'))!.name;

          compilation.updateAsset(cssFilename, (old) => {
            const oldSource = old.source().toString();
            expect(
              oldSource.includes('url("../../static/svg/logo.svg")'),
            ).toBeTruthy();
            return old;
          });
        },
      );
    });
  }
}

export default defineConfig({
  entry: './index.js',
  target: 'web',
  plugins: [new Plugin()],
  output: {
    cssFilename: 'static/initial/index.css',
  },
  module: {
    rules: [
      {
        test: /\.svg$/,
        type: 'asset/resource',
        generator: {
          filename: 'static/svg/[name][ext]',
        },
      },
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
});
