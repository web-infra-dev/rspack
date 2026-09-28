import type { Compiler } from '@rspack/core';
import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

class Plugin {
  apply(compiler: Compiler) {
    compiler.hooks.thisCompilation.tap('TestFakePlugin', (compilation) => {
      compilation.hooks.runtimeModule.tap('TestFakePlugin', (module) => {
        if (module.name !== 'css_loading' || !module.source) return;
        const originCode = module.source.source.toString('utf-8');

        module.source.source = Buffer.from(
          originCode
            .replace(
              /document\.(getElementsByTagName|querySelectorAll)/g,
              `(globalThis.APP_ROOT||document).$1`,
            )
            .replace(
              /document\.head/g,
              `(globalThis.APP_STYLE_ROOT||document.head)`,
            ),
          'utf-8',
        );
      });
    });
  }
}

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  target: 'web',
  mode: 'development',
  node: {
    __dirname: false,
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/module',
      },
    ],
  },
  plugins: [new Plugin(), new rspack.HotModuleReplacementPlugin()],
});
