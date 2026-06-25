const { rspack } = require('@rspack/core');

class Plugin {
  apply(compiler) {
    compiler.hooks.compilation.tap('TestFakePlugin', (compilation) => {
      compilation.hooks.runtimeModule.tap('TestFakePlugin', (module, chunk) => {
        if (module.constructor.name === 'CssLoadingRuntimeModule') {
          expect(module.identifier()).toBe('webpack/runtime/css loading');
          expect(module.readableIdentifier()).toBe(
            'webpack/runtime/css loading',
          );
          const originSource = module.source.source.toString('utf-8');
          let runtimeSource;
          if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
            runtimeSource = '__rspack_context.f.miniCss.test = true;';
          } else {
            runtimeSource = '__webpack_require__.f.miniCss.test = true;';
          }
          module.source.source = Buffer.from(
            `${originSource}\n${runtimeSource}\n`,
            'utf-8',
          );
        }
      });
    });
  }
}

/**@type {import("@rspack/core").Configuration}*/
module.exports = {
  target: 'web',
  module: {
    rules: [
      {
        test: /\.css$/,
        use: [rspack.CssExtractRspackPlugin.loader, 'css-loader'],
        type: 'javascript/auto',
      },
    ],
  },
  plugins: [new rspack.CssExtractRspackPlugin(), new Plugin()],
};
