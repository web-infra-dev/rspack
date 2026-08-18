const { DefinePlugin } = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  plugins: [
    new DefinePlugin({
      __CJS_COLLISION_DEFINE__: '__RSPACK_CJS_EXPORT_defined__',
      __CJS_ANONYMOUS_DEFINE__: 'function () {}',
    }),
  ],
};
