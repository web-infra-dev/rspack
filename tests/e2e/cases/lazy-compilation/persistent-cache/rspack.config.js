const { rspack } = require('@rspack/core');

/** @type { import('@rspack/core').RspackOptions } */
module.exports = {
  context: __dirname,
  entry: {
    main: ['./src/component.js', './src/index.js'],
  },
  stats: 'none',
  mode: 'production',
  plugins: [
    new rspack.HtmlRspackPlugin(),
    {
      apply(compiler) {
        compiler.hooks.done.tap('TEST', function (stats) {
          const { modules } = stats.toJson({ modules: true });
          compiler.__modules = modules.map((item) => item.identifier);
        });
      },
    },
  ],
  lazyCompilation: true,
  cache: {
    type: 'persistent',
  },
  devServer: {
    hot: true,
    client: {
      overlay: {
        // hide warnings for incremental
        warnings: false,
      },
    },
  },
};
