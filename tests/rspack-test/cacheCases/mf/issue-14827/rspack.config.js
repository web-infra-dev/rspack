const rspack = require('@rspack/core');

let compilerIndex = 0;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  context: __dirname,
  optimization: {
    minimize: false,
  },
  cache: {
    type: 'persistent',
  },
  plugins: [
    {
      apply(compiler) {
        if (compilerIndex > 0) {
          compiler.options.cache.readonly = true;
          new rspack.container.ModuleFederationPlugin({
            shared: ['./shared'],
          }).apply(compiler);
        }
        compilerIndex++;
      },
    },
  ],
};
