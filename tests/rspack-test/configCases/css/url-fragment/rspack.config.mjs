/** @type {import("@rspack/core").Configuration[]} */
export default ['css', 'css/auto', 'css/module'].map((type) => ({
  target: 'web',
  mode: 'development',
  devtool: false,
  output: {
    publicPath: '',
    assetModuleFilename: '[name][ext][fragment]',
  },
  module: {
    rules: [
      { test: /\.css$/, type },
      { test: /\.svg$/, type: 'asset/resource' },
    ],
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.compilation.tap('Test', (compilation) => {
          compilation.hooks.finishModules.tap('Test', (modules) => {
            const requests = Array.from(modules)
              .filter((module) => module.type.startsWith('css'))
              .flatMap((module) =>
                module.dependencies.map((dependency) => dependency.request),
              )
              .filter(Boolean);
            expect(requests).toEqual([
              './filters.svg#myFilter',
              './filters.svg#myFilter',
            ]);
          });
        });
      },
    },
  ],
}));
