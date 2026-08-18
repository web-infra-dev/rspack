const { DefinePlugin } = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  optimization: {
    concatenateModules: true,
  },
  plugins: [
    new DefinePlugin({
      DEFINED_STRING: JSON.stringify('__rspack_module_ref0__._'),
    }),
  ],
};
