const { DefinePlugin } = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  plugins: [
    new DefinePlugin({
      __CJS_COLLISION_DEFINE__: '__RSPACK_CJS_EXPORT_defined__',
      __CJS_ESCAPED_COLLISION_DEFINE__: String.raw`\u005f\u005fRSPACK_CJS_EXPORT_escapedDefined__`,
      __CJS_ANONYMOUS_DEFINE__: 'function () {}',
    }),
  ],
};
