/** @type {import('webpack').Configuration} */
export default {
  entry: 'data:text/javascript,import "./index.js";',
  plugins: [
    function (compiler) {
      compiler.hooks.compilation.tap(
        'test',
        (compilation, { normalModuleFactory }) => {
          normalModuleFactory.hooks.afterResolve.tap('test', () => {});
        },
      );
    },
  ],
};
