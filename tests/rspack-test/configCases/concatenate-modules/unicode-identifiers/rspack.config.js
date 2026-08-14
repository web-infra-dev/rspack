const { DefinePlugin } = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  optimization: {
    concatenateModules: true,
  },
  plugins: [
    new DefinePlugin({
      GENERATED_UNICODE: 'typeof 原始拼接全局引用',
      GENERATED_ESCAPED_UNICODE: String.raw`typeof \u8f6c\u4e49\u62fc\u63a5\u5168\u5c40\u5f15\u7528`,
    }),
  ],
};
